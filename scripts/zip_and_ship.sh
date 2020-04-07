#!/bin/bash -eu

WORKING_DIR=$HOME/.cbuilder

# This script requires the registry name to be set
if [ -z "${REGISTRY_NAME:-}" ]; then
    echo "REGISTRY_NAME needs to be set"
    exit 1
fi

TARGET_ACCOUNT=
# If other account is set then use that otherwise use the root account
if [ -z "${TARGET:-}" ]; then
    # Get the account id from properties
    TARGET_ACCOUNT=$(grep ROOT_ACCOUNT_ID "$WORKING_DIR"/properties | cut -d '=' -f2)
else
    TARGET_ACCOUNT="$TARGET"
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
    # Setup access
    echo "aws configure set profile.target_profile.role_arn arn:aws:iam::$TARGET_ACCOUNT:role/ContainerBuilderPushRole"
    echo "aws configure set profile.target_profile.credential_source Ec2InstanceMetadata"
    # Push the container
    echo "aws ecr get-login-password --profile target_profile --region us-east-1 | docker login --username AWS --password-stdin $REGISTRY_NAME"
    echo "docker build -t $REGISTRY_NAME $@ ."
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
