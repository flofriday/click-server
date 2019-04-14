# select image
FROM rust:1.34-slim

# copy your source tree (but not the dynamic folder, since it will be mounted with a bind mount)
COPY ./ ./
RUN rm -r dynamic

# build for release
RUN cargo build --release

# expose the port 8000
EXPOSE 8000

# set the startup command to run your binary
CMD ["./target/release/click"]