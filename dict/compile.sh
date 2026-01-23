gcc -o examples/example_c examples/example.c \
    -L./target/release -ldict \
    -Wl,-rpath=./target/release