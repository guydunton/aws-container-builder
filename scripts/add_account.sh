#!/bin/bash -eu

if [ -z ${NEW_ACCOUNT_PROFILE:-} ]; then
    echo "NEW_ACCOUNT_PROFILE must be set"
    exit 1
fi

if [ -z ${BASE_PROFILE:-} ]; then
    echo "BASE_PROFILE must be set"
    exit 1
fi

WORKING_DIR=$HOME/.cbuilder

BASE_ACCOUNT=$(aws sts get-caller-identity \
    --profile "$BASE_PROFILE" \
    --query 'Account' \
    --output text)
NEW_ACCOUNT=$(aws sts get-caller-identity \
    --profile "$NEW_ACCOUNT_PROFILE" \
    --query 'Account' \
    --output text)

NEW_ACCOUNT_STACK_NAME=container-builder-role

# Deploy the role stack to the new account
echo "Deploying role to new account"
aws cloudformation deploy \
    --stack-name "$NEW_ACCOUNT_STACK_NAME" \
    --template-file "$WORKING_DIR"/resources/role-cfn.yml \
    --capabilities CAPABILITY_NAMED_IAM \
    --tags \
        Squad=test@test.com \
        Name=container-builder-role \
        Compliance=no \
    --parameter-overrides \
        RootAccount="$BASE_ACCOUNT" \
    --profile "$NEW_ACCOUNT_PROFILE" \
> /dev/null
echo "Deployed role to new account"

# Create policy document
sed "s/{ACCOUNT}/$NEW_ACCOUNT/g" "$WORKING_DIR"/resources/role_policy.template.json > /tmp/policy.json

# Deploy the extra policy to our root account
echo "Deploying policy to root account"
aws iam put-role-policy \
    --profile "$BASE_PROFILE" \
    --role-name ContainerBuilderRole \
    --policy-name container-access-"$NEW_ACCOUNT" \
    --policy-document file:///tmp/policy.json
echo "Deployed policy to root account"
