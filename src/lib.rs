use arma_rs::{arma, Extension};
use std::ffi::CString;
use windows::core::{Interface as _, PCSTR};
use windows::Win32::Graphics::Direct3D11::*;
use windows::Win32::Graphics::Dxgi::Common::{
    DXGI_FORMAT_B8G8R8A8_UNORM, DXGI_FORMAT_R8G8B8A8_UNORM,
};
use windows::Win32::System::LibraryLoader::{GetModuleHandleA, GetProcAddress};

#[arma]
fn init() -> Extension {
    Extension::build().command("take", take).finish()
}

#[repr(C)]
#[derive(Copy, Clone)]
struct RVExtensionRenderInfo {
    d3d_device: *mut std::ffi::c_void,         // ID3D11Device*
    d3d_device_context: *mut std::ffi::c_void, // ID3D11DeviceContext*
}
type RVExtensionGLockProc = unsafe extern "C" fn() -> *mut RVExtensionGraphicsLockGuard;

#[repr(C)]
struct RVExtensionGraphicsLockGuard {
    vtable: *const RVExtensionGraphicsLockGuardVTable,
}

#[repr(C)]
struct RVExtensionGraphicsLockGuardVTable {
    release_lock: unsafe extern "C" fn(*mut RVExtensionGraphicsLockGuard),
}

fn take() -> Option<()> {
    let device_data_ptr =
        unsafe { find_rv_function("RVExtensionGData") } as *const *const RVExtensionRenderInfo;
    let gpu_lock_fn = unsafe { find_rv_function("RVExtensionGLock") } as *const ();

    if device_data_ptr.is_null() || gpu_lock_fn.is_null() {
        return None;
    }

    let lock_guard = unsafe {
        let lock_fn: RVExtensionGLockProc = std::mem::transmute(gpu_lock_fn);
        lock_fn()
    };

    let render_info = unsafe { **device_data_ptr };

    let device: ID3D11Device = unsafe {
        let ptr = render_info.d3d_device as *mut ID3D11Device;
        std::mem::transmute_copy(&ptr)
    };

    let context: ID3D11DeviceContext = unsafe {
        let ptr = render_info.d3d_device_context as *mut ID3D11DeviceContext;
        std::mem::transmute_copy(&ptr)
    };

    unsafe {
        let mut render_targets = [None];
        context.OMGetRenderTargets(Some(&mut render_targets), None);

        let render_target = render_targets[0].as_ref()?;
        let backbuffer: ID3D11Texture2D = render_target.GetResource().ok()?.cast().ok()?;

        let mut desc = D3D11_TEXTURE2D_DESC::default();
        backbuffer.GetDesc(&mut desc);

        let staging_desc = D3D11_TEXTURE2D_DESC {
            Usage: D3D11_USAGE_STAGING,
            CPUAccessFlags: D3D11_CPU_ACCESS_READ.0 as u32,
            ..desc
        };

        let mut staging: Option<ID3D11Texture2D> = None;
        device
            .CreateTexture2D(&staging_desc, None, Some(&mut staging))
            .map_err(|e| {
                println!("CreateTexture2D failed: {:?}", e);
                e
            }).ok()?;
        let staging = staging?;

        context.CopyResource(&staging, &backbuffer);

        let mut mapped: D3D11_MAPPED_SUBRESOURCE = std::mem::zeroed();
        context
            .Map(&staging, 0, D3D11_MAP_READ, 0, Some(&mut mapped as *mut _))
            .ok()?;

        let row_pitch = mapped.RowPitch as usize;
        let width = desc.Width as usize;
        let height = desc.Height as usize;
        let data =
            std::slice::from_raw_parts(mapped.pData as *const u8, row_pitch * height).to_vec();

        context.Unmap(&staging, 0);

        let mut rgb_data = Vec::with_capacity(width * height * 3);
        for chunk in data.chunks(4) {
            if chunk.len() >= 4 {
                match desc.Format {
                    DXGI_FORMAT_B8G8R8A8_UNORM => {
                        // BGRA: B, G, R, A
                        rgb_data.push(chunk[2]); // R
                        rgb_data.push(chunk[1]); // G
                        rgb_data.push(chunk[0]); // B
                    }
                    DXGI_FORMAT_R8G8B8A8_UNORM => {
                        // RGBA: R, G, B, A
                        rgb_data.push(chunk[0]); // R
                        rgb_data.push(chunk[1]); // G
                        rgb_data.push(chunk[2]); // B
                    }
                    _ => {
                        return None;
                    }
                }
            }
        }

        let img = image::RgbImage::from_raw(width as u32, height as u32, rgb_data)?;
        let filename = "screenshot.jpg";
        img.save(filename).ok()?;
    };

    unsafe {
        if !lock_guard.is_null() {
            let vtable = (*lock_guard).vtable;
            ((*vtable).release_lock)(lock_guard);
        }
    }

    Some(())
}

unsafe fn find_rv_function(name: &str) -> *const () {
    let cname = CString::new(name).unwrap();

    let Ok(handle) = GetModuleHandleA(None) else {
        return std::ptr::null();
    };

    GetProcAddress(handle, PCSTR(cname.as_ptr() as *const u8))
        .map(|p| p as *const ())
        .unwrap_or(std::ptr::null())
}
