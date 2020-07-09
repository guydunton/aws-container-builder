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
