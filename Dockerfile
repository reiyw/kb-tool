FROM quay.io/pypa/manylinux1_x86_64

ENV PATH /root/.cargo/bin:$PATH
# Add all supported python versions
ENV PATH /opt/python/cp35-cp35m/bin/:/opt/python/cp36-cp36m/bin/:/opt/python/cp37-cp37m/bin/:$PATH
# Otherwise `cargo new` errors
ENV USER root

RUN curl https://sh.rustup.rs -sSf | \
    sh -s -- --default-toolchain nightly -y \
    && python3 -m pip install --no-cache-dir cffi maturin \
    && mkdir /io

WORKDIR /io

ENTRYPOINT ["maturin"]
