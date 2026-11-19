#ifndef VERSION_H_
#define VERSION_H_


#include <stdio.h>

// Phien ban fork NCKH - Ferb27
#ifndef DE_FORK
#define DE_FORK 1
#endif

#ifndef __APP__VERSION__
#define __APP__VERSION__ "5.7.0-fork"
#endif

static std::string version_string = __APP__VERSION__; 
#endif
