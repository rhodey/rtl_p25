FROM rust:1-buster

RUN apt update
RUN apt install -y \
  libusb-1.0.0 \
  libusb-1.0.0-dev \
  libsndfile1 \
  libsndfile1-dev \
  clang \
  libclang-dev \
  cmake

ENV HOME=/root/app
RUN mkdir -p ${HOME}/include
WORKDIR $HOME/include

RUN git clone -b rw --single-branch --depth=1 https://github.com/rhodey/librtlsdr
RUN mkdir -p include/librtlsdr/build
WORKDIR ${HOME}/include/librtlsdr/build
RUN cmake ../ -DINSTALL_UDEV_RULES=ON -DDETACH_KERNEL_DRIVER=ON
RUN make
RUN make install

WORKDIR ${HOME}/include
RUN git clone -b master --single-branch --depth=1 https://github.com/rhodey/rtl_rs
WORKDIR ${HOME}/include/rtl_rs
RUN rustup override add nightly
RUN cargo build --release
RUN cargo install --force --path .

WORKDIR ${HOME}/include
RUN git clone --depth=1 https://github.com/rhodey/liquid-dsp
WORKDIR ${HOME}/include/liquid-dsp
RUN chmod +x bootstrap.sh
RUN ./bootstrap.sh && ./configure
RUN make
RUN make install
RUN ldconfig || true

WORKDIR $HOME
COPY Cargo.toml Cargo.toml
COPY Cargo.lock Cargo.lock
COPY liquid_dsp_rs/ liquid_dsp_rs/
COPY src/ src/

RUN cargo build --target-dir liquid_dsp_rs
RUN cargo install --force --path .

ENTRYPOINT ["target/release/rtl_p25"]
