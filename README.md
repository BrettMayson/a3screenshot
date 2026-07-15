# Screenshot Example

This code demonstrates taking a screenshot of Arma 3 with the DirectX 11 API. It captures the back buffer, converts it to RGB format, and saves it as a JPEG file.

This is something the `screenshot` command isn't capable of doing on Linux, for some reason.

```sqf
"screenshot" callExtension ["take", []]
```

I have only tested this on Linux with `cargo build --release --target x86_64-pc-windows-gnu`, but I see no reason Windows would struggle to run Windows code.
