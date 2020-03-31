#!/bin/bash -eu

STACK_NAME=container-builder
SSH_KEY_NAME=ContainerBuilderKey

if [ -z "${AWS_PROFILE:-}" ]; then
    echo "AWS_PROFILE must be set"
    exit 1
fi

WORKING_DIR="$HOME/.cbuilder"

# Conditionally create the .cbuilder directory
if [ ! -d "$WORKING_DIR" ]; then
    mkdir "$WORKING_DIR"
fi

# Generate an SSH key for new instance
SSH_KEY=$(aws ec2 create-key-pair \
    --key-name "$SSH_KEY_NAME" \
    --query 'KeyMaterial' \
    --output text)

# Store the ssh key
echo "$SSH_KEY" > "$WORKING_DIR/$SSH_KEY_NAME".pem
chmod 400 "$WORKING_DIR/$SSH_KEY_NAME".pem
echo "Created SSH key"

# Find the amazon linux 2 AMI
AMI=$(aws ec2 describe-images \
    --owners amazon \
    --filters 'Name=name,Values=amzn2-ami-hvm-2.0.????????.?-x86_64-gp2' 'Name=state,Values=available' \
    --query 'reverse(sort_by(Images, &CreationDate))[:1].ImageId' \
    --output text)

# Deploy the cloudformation stack
echo "Deploying instance stack to AWS"
aws cloudformation deploy \
    --stack-name "$STACK_NAME" \
    --template-file "$WORKING_DIR"/resources/instance-cfn.yml \
    --capabilities CAPABILITY_NAMED_IAM \
    --tags \
        Squad=test@test.com \
        Name=container-builder \
        Compliance=no \
    --parameter-overrides \
        AmiId="$AMI" \
        SSHKeyName="$SSH_KEY_NAME" \
> /dev/null

echo "Finished deploying instance to AWS"

# Get the instance IP
INSTANCE_IP=$(aws cloudformation describe-stacks \
    --stack-name "$STACK_NAME" \
    --query "Stacks[*].Outputs[?OutputKey=='InstanceIP'].OutputValue" \
    --output text)

# Create a connect bash script
{
    echo "#!/bin/bash"
    echo "ssh -i $WORKING_DIR/$SSH_KEY_NAME.pem ec2-user@$INSTANCE_IP"
} > "$WORKING_DIR/scripts/connect.sh"
chmod +x "$WORKING_DIR/scripts/connect.sh"
echo "Created connect script"

# Create a details file
{
    echo "SSH_KEY=$WORKING_DIR/$SSH_KEY_NAME.pem"
    echo "INSTANCE_IP=$INSTANCE_IP"
} > "$WORKING_DIR/properties"
echo "Created "

# Create an uninstall bash script
{
    echo "#!/bin/bash"
    echo "echo 'Deleting all resources'"
    echo "chmod 600 $WORKING_DIR/$SSH_KEY_NAME.pem"
    echo "rm $WORKING_DIR/$SSH_KEY_NAME.pem"
    echo "aws ec2 delete-key-pair --key-name $SSH_KEY_NAME --profile $AWS_PROFILE"
    echo "echo 'Deleted Key'"
    echo "rm $WORKING_DIR/properties"
    echo "echo 'Deleted properties'"
    echo "echo 'Starting stack deletion'"
    echo "aws cloudformation delete-stack --stack-name $STACK_NAME --profile $AWS_PROFILE"
    echo "aws cloudformation wait stack-delete-complete --stack-name $STACK_NAME --profile $AWS_PROFILE"
    echo "echo 'Stack delete complete'"
    echo "rm $WORKING_DIR/scripts/connect.sh"
    echo "echo 'Deleted all scripts'"
} > "$WORKING_DIR/scripts/uninstall.sh"
chmod +x "$WORKING_DIR/scripts/uninstall.sh"
echo "Created uninstall script"

echo "Bootstrap complete"
