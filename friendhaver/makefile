EXE = mexx

default: pgo-build

pgo-build:
	# STEP 0: Make sure there is no left-over profiling data from previous runs
	rm -rf /tmp/pgo-data

	# STEP 1: Build the instrumented binaries
	RUSTFLAGS="-Cprofile-generate=/tmp/pgo-data" \
    	cargo build --release

	# STEP 2: Run the instrumented binaries with some typical data
	./target/release/mexx bench

	# STEP 3: Merge the `.profraw` files into a `.profdata` file
	llvm-profdata merge -o /tmp/pgo-data/merged.profdata /tmp/pgo-data

	# STEP 4: Use the `.profdata` file for guiding optimizations
	RUSTFLAGS="-Cprofile-use=/tmp/pgo-data/merged.profdata" \
    	cargo build --release
	
	# STEP 5: Move the built binary to the final location
	mv ./target/release/mexx $(EXE)
