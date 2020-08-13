FROM ubuntu:20.04

# [Optional] Uncomment this section to install additional OS packages.
# RUN apt-get update && export DEBIAN_FRONTEND=noninteractive \
#     && apt-get -y install --no-install-recommends <your-package-list-here>
LABEL maintainer="Hattomo"

RUN \
  apt-get update && apt-get -y upgrade && \
  apt-get install -y nano sudo openssl less git && \
  apt-get autoremove -y && apt-get clean -y

ARG DOCKER_UID=1002
ARG DOCKER_USER=hattomo
ARG DOCKER_PASSWORD=0514
RUN useradd -m --uid ${DOCKER_UID} --groups sudo ${DOCKER_USER} \
  && echo ${DOCKER_USER}:${DOCKER_PASSWORD} | chpasswd

# change user to ${DOCKER_USER}
USER ${DOCKER_USER}