# select image
FROM rust:1.33

# copy your source tree
COPY ./ ./
RUN rm click.txt

# build for release
RUN cargo build --release

# expose the port 8000
EXPOSE 8000

# set the startup command to run your binary
CMD ["./target/release/click"]