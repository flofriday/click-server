# click-server
Is a simple web-server written in rust.
The main purpose of this project is to learn deployment.

## Build it yourself
1. Copy the repostory and [install rust](https://www.rust-lang.org/tools/install)
2. Run `cargo run` in your terminal
3. Open [localhost:8000](http://localhost:8000) in your browser

## Run with docker
1. Create a empty file on your system `touch clicks.txt`
2. Build the image with `sudo docker build -t click-server .`
3. Run the image with `sudo docker run -d --rm -p 8000:8000 --mount type=bind,source="$(pwd)"/clicks.txt,target=/clicks.txt --name click-server1 click-server`
4. Open [localhost:8000](http://localhost:8000) in your browser

## Credits
[background.jpeg](https://github.com/flofriday/click-server/blob/master/static/background.jpg) - [Josh Spires](https://unsplash.com/@drone_nr)