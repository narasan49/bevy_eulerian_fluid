#define_import_path euler_fluid_3d::workgroup_shape

#ifdef WG8X8X1
const WG_SIZE: vec3u = vec3u(8, 8, 1);
#else ifdef WG8X8X4
const WG_SIZE: vec3u = vec3u(8, 8, 4);
#else
const WG_SIZE: vec3u = vec3u(8, 8, 4);
#endif