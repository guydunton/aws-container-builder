# AWS Container Builder

AWS container builder is a tool for speeding up builds of Docker containers by using AWS instances.

Code is sent to an instances which runs `docker build` and then uploads the container to a registry. Filters in `.dockerignore` files will be used to reduce bandwidth being used.

## How to install

Checkout the code into the ~/.cbuilder directory using the following command:

```bash
git clone https://github.com/guydunton/aws-container-builder.git ~/.cbuilder
```

You should also add the `~/.cbuilder` repository into your path.

## Usage

Assuming that `~/.cbuilder` is in your path you can run the following command to see the help text:

```bash
builder --help
```

### Setup environment

To create a VM in AWS run the command `builder bootstrap`. This will create a VM, SSH key and setup all necessary scripts to connect to the VM.

**Note** The `builder bootstrap` command requires `AWS_PROFILE` to be set.

### Build a docker container

To build a docker container remotely cd into the project you wish to build containing a `Dockerfile`. You must already have an ECR (elastic container registry) setup.

To start the build run the command `builder ship REGISTRY` replacing `REGISTRY` with the URI of the registry to push the resulting container to. e.g. `builder ship 012345678987.dkr.ecr.eu-west-1.amazonaws.com/test`.

### Cleanup the environment

To remove the VM and cleanup the SSH keys run the following command `builder cleanup`. To completely remove container builder from your machine you can also delete the `~/.cbuilder` folder.

## Local development

To run the project locally run the `local_deploy.sh` script which will copy all files into `$HOME/.cbuilder/`.
