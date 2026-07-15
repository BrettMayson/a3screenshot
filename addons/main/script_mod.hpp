#define MAINPREFIX z
#define PREFIX scr

#include "script_version.hpp"

#define VERSION MAJOR.MINOR.PATCH
#define VERSION_AR MAJOR,MINOR,PATCH

#define REQUIRED_VERSION 2.20

#ifdef COMPONENT_BEAUTIFIED
    #define COMPONENT_NAME QUOTE(Screenshot - COMPONENT_BEAUTIFIED)
#else
    #define COMPONENT_NAME QUOTE(Screenshot - COMPONENT)
#endif
