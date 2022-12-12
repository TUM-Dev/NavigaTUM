#!/usr/bin/bash

# args: "main"
# args: non-main-branch PR-number

set -e # fail on first
tmp_dir=$(mktemp -d -t navigatum-XXXXXXXXXX)
# remove the tmp dir on exit
#trap "trap - SIGTERM && rm -fr $tmp_dir" SIGINT SIGTERM EXIT
echo "Temporary files will be written to: $tmp_dir"

if [ $1 == "main" ]; then
    echo "Deploying to production"
    export TARGET=main

    rm ./deployment/k3s/templates/application.yaml

    mkdir "$tmp_dir/$TARGET"
    cp -R ./deployment/k3s/* "$tmp_dir/main"
else
    echo "Deploying to PR-based environment"

    export BRANCH=$1
    export PR_NUMBER=$2
    export TARGET=pr-$PR_NUMBER
    echo "creating app for $BRANCH on $TARGET"

    # In PR-based environments, someone knowing the Meili master key is not a concern
    MEILI_MASTER_KEY_RAW="$(echo $RANDOM | md5sum | head -c 20)$(echo $RANDOM | md5sum | head -c 20)$(echo $RANDOM | md5sum | head -c 20)"
    MEILI_MASTER_KEY="$(echo "$MEILI_MASTER_KEY_RAW"|base64)"
    export MEILI_MASTER_KEY

    mkdir "$tmp_dir/$TARGET"
    cat ./deployment/k3s/templates/application.yaml | kyml tmpl --env BRANCH --env PR_NUMBER --env MEILI_MASTER_KEY > tmp.yaml
    cp -R ./deployment/k3s/* "$tmp_dir/$TARGET"
    mv tmp.yaml "$tmp_dir/$TARGET/templates/application.yaml"
fi

echo "checking out deployment branch"
git checkout deployment --
rm -fr "./$TARGET"
mv $tmp_dir/* .
git add .
echo "current git status"
git status

# --porcelain returns nothing if there are no changes
if [ -z "$(git status --porcelain)" ]; then
    echo "no changes to deployment, returnning"
    exit 0
fi

# git branch deployment
if [ $1 == "main" ]; then
    git commit -m "Updated the production deployment"
else
    git commit -m "Updated the staging deployment for pr-$2"
fi

git push
