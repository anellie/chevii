fn main() {
    cxx_build::bridge("src/ai/nnue.rs")
        .file("src/ai/nnue/nnue.cpp")
        .file("src/ai/nnue/misc.cpp")
        .flag("-Wall")
        .flag("-fstrict-aliasing")
        .flag("-fno-exceptions")
        .flag("-fno-rtti")
        .flag("-std=c++11")
        .flag("-Ofast")
        .flag("-fomit-frame-pointer")
        .flag("-fpermissive")
        .compile("chevii");
}
