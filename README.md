# Tortoise

The goal of this crate is to translate compute shaders in SPIR-V format into Rust code that can be executed on CPU. The main use case is to be able to run the [piet-gpu] shader pipeline even on hardware with inadequate or obsolete GPU infrastructure.

There are a number of projects with similar scope. The most ambitious are [SwiftShader] and Mesa [Lavapipe], which run a significant fraction of all of Vulkan. However, they are based on doing JIT translation at runtime, which is a significant dependency and impacts startup time.

Probably the closest existing work is [glsl-to-cxx], which translates GLSL to C++ for use in WebRender. There are a few differences; we start with SPIR-V, to leverage existing shader compilation infrastructure (and to reduce friction for source languages other than GLSL). We target Rust mostly because it has standard SIMD support, and also to minimize non-Rust tooling dependencies. And we target compute shaders rather than vertex+fragment, because that's what piet-gpu uses.

Another related project is the [spirv to ispc translator]. This is based on an earlier version of spirv-cross, which had C++ output (now removed).

A significant amount of work in OpenCL is also geared to being able to run workloads on CPU, as is [Android Renderscript].

The first milestone for this crate would be to generate fairly simplistic scalar code, to get the shaders running at all. A further direction would be to map it to SIMD code, using many of the techniques of [ispc]. Major techniques would include detection of uniform, detection of linear memory access patterns (so SIMD load/store intrinsics could be used), and organization of access to workgroup-shared memory around barriers.

## License

The license is MIT or Apache 2.0, at your choice.

[piet-gpu]: https://github.com/linebender/piet-gpu
[SwiftShader]: https://swiftshader.googlesource.com/SwiftShader
[Lavapipe]: https://www.phoronix.com/scan.php?page=news_item&px=Mesa-Vulkan-Lavapipe
[glsl-to-cxx]: https://github.com/servo/webrender/tree/master/glsl-to-cxx
[spirv to ispc translator]: https://software.intel.com/content/www/us/en/develop/articles/spir-v-to-ispc-convert-gpu-compute-to-the-cpu.html
[ispc]: https://ispc.github.io/
[Android Renderscript]: https://developer.android.com/guide/topics/renderscript/compute
