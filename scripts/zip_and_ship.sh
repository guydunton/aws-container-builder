#!/bin/bash -eu

WORKING_DIR=$HOME/.cbuilder

# This script requires the registry name to be set
if [ -z "${REGISTRY_NAME:-}" ]; then
    echo "REGISTRY_NAME needs to be set"
    exit 1
fi

# Find a list of all files in the current directory (excluding .dockerignore files)
files=$(python3 "$WORKING_DIR"/scripts/find_files_in_dir.py)

# Create archive
tar -czf /tmp/archive.tar.gz $files > /dev/null
echo "Created archive"

# Get the ssh key & IP
KEY=$(grep SSH_KEY "$WORKING_DIR"/properties | cut -d '=' -f2)
IP=$(grep INSTANCE_IP "$WORKING_DIR"/properties | cut -d '=' -f2)

# Copy the archive over to the remote machine
scp -i "$KEY" /tmp/archive.tar.gz ec2-user@"$IP":/home/ec2-user > /dev/null
echo "Transfered archive to remote machine"

# Remove the local copy of the archive
rm /tmp/archive.tar.gz

# Create a script to run on the remote machine
{
    # Extract the archive
    echo "#!/bin/bash -eux"
    echo "mkdir archive"
    echo "tar -xzvf archive.tar.gz -C archive"
    # Build the docker container
    echo "cd archive"
    # Push the container
    echo "$(aws ecr get-login --region us-east-1 --no-include-email)"
    echo "docker build -t $REGISTRY_NAME ."
    echo "docker push $REGISTRY_NAME:latest"
    # Remove the archive
    echo "cd .." 
    echo "rm -rf archive"
} > /tmp/remote-script.sh
chmod +x /tmp/remote-script.sh
echo "Created build script"

# Copy the script onto the remote machine
scp -i "$KEY" /tmp/remote-script.sh ec2-user@"$IP":/home/ec2-user
echo "Copied build script to remote machine"

# Delete the local copy of the script
rm /tmp/remote-script.sh

# Run the script on the remote machine
ssh -i "$KEY" ec2-user@"$IP" './remote-script.sh > log'
