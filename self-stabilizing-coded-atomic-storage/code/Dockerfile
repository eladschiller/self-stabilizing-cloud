FROM ubuntu:18.04
LABEL Description="Docker image for CASSS servers"

COPY . /opt/project
WORKDIR /opt/project

# Install base system required for build
RUN apt-get update &&\
    apt-get install -y \
    gcc g++ musl-dev bash software-properties-common \
    curl net-tools iperf3 iputils-ping \
    python3.6-dev python3-pip

# Install dependencies
RUN apt-get install -y python3-zmq \
    liberasurecode-dev libjerasure-dev
RUN pip3 install -r requirements.txt

EXPOSE 5555

# CMD ["python3.6", "./server.py", "5555", "$(hostname -I)", "./config/docker.ini"]
