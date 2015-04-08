import platform
plat = platform.system()

envspooky = Environment(CPPPATH = ['deps/spookyhash/'], CPPFLAGS="-fno-exceptions -O2")
spooky = envspooky.Library('spooky', Glob("deps/spookyhash/*.cpp"))

envmurmur = Environment(CPPPATH = ['deps/murmurhash/'], CPPFLAGS="-fno-exceptions -O2")
murmur = envmurmur.Library('murmur', Glob("deps/murmurhash/*.cpp"))

envbloom = Environment(CCFLAGS = '-std=c99 -Wall -Werror -Wextra -O2 -D_GNU_SOURCE')
bloom = envbloom.Library('bloom', Glob("csrc/libbloom/*.c"), LIBS=[murmur, spooky])

envtest = Environment(CCFLAGS = '-std=c99 -Wall -Werror -Wextra -Wno-unused-function -D_GNU_SOURCE -Icsrc/libbloom/')
envtest.Program('test_libbloom_runner', Glob("tests/libbloom/*.c"), LIBS=["check", bloom, murmur, spooky, "m"])

envinih = Environment(CPATH = ['deps/inih/'], CFLAGS="-O2")
inih = envinih.Library('inih', Glob("deps/inih/*.c"))

envbloomd_with_err = Environment(CCFLAGS = '-std=c99 -D_GNU_SOURCE -Wall -Wextra -Werror -O2 -pthread -Icsrc/bloomd/ -Ideps/inih/ -Ideps/libev/ -Icsrc/libbloom/')
envbloomd_without_unused_err = Environment(CCFLAGS = '-std=c99 -D_GNU_SOURCE -Wall -Wextra -Wno-unused-function -Wno-unused-result -Werror -O2 -pthread -Icsrc/bloomd/ -Ideps/inih/ -Ideps/libev/ -Icsrc/libbloom/')
envbloomd_without_err = Environment(CCFLAGS = '-std=c99 -D_GNU_SOURCE -O2 -pthread -Icsrc/bloomd/ -Ideps/inih/ -Ideps/libev/ -Icsrc/libbloom/')

objs =  envbloomd_with_err.Object('csrc/bloomd/config', 'csrc/bloomd/config.c') + \
        envbloomd_without_err.Object('csrc/bloomd/networking', 'csrc/bloomd/networking.c') + \
        envbloomd_with_err.Object('csrc/bloomd/barrier', 'csrc/bloomd/barrier.c') + \
        envbloomd_with_err.Object('csrc/bloomd/conn_handler', 'csrc/bloomd/conn_handler.c') + \
        envbloomd_with_err.Object('csrc/bloomd/filter', 'csrc/bloomd/filter.c') + \
        envbloomd_with_err.Object('csrc/bloomd/filter_manager', 'csrc/bloomd/filter_manager.c') + \
        envbloomd_with_err.Object('csrc/bloomd/background', 'csrc/bloomd/background.c') + \
        envbloomd_with_err.Object('csrc/bloomd/art', 'csrc/bloomd/art.c')

bloom_libs = ["pthread", bloom, murmur, inih, spooky, "m"]
if plat == 'Linux':
   bloom_libs.append("rt")

bloomd = envbloomd_with_err.Program('bloomd', objs + ["csrc/bloomd/bloomd.c"], LIBS=bloom_libs)

if plat == "Darwin":
    bloomd_test = envbloomd_without_err.Program('test_bloomd_runner', objs + Glob("tests/bloomd/runner.c"), LIBS=bloom_libs + ["check"])
else:
    bloomd_test = envbloomd_without_unused_err.Program('test_bloomd_runner', objs + Glob("tests/bloomd/runner.c"), LIBS=bloom_libs + ["check"])

bench_obj = Object("bench", "bench.c", CCFLAGS="-std=c99 -O2")
Program('bench', bench_obj, LIBS=["pthread"])

# By default, only compile bloomd
Default(bloomd)
