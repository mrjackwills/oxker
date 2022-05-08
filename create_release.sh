#!/bin/bash

# rust create_release
# v0.0.14

PACKAGE_NAME='oxker'
STAR_LINE='****************************************'
CWD=$(pwd)

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[0;33m'
PURPLE='\033[0;35m'
RESET='\033[0m'


# $1 string - error message
error_close() {
	echo -e "\n${RED}ERROR - EXITED: ${YELLOW}$1${RESET}\n";
	exit 1
}


if [ -z "$PACKAGE_NAME" ]
then
	error_close "No package name"
fi

# $1 string - question to ask
ask_yn () {
	printf "%b%s? [y/N]:%b " "${GREEN}" "$1" "${RESET}"
}

# return user input
user_input() {
	read -r data
	echo "$data"
}

update_major () {
	local bumped_major
	bumped_major=$((MAJOR + 1))
	echo "${bumped_major}.0.0"
}

update_minor () {
	local bumped_minor
	bumped_minor=$((MINOR + 1))
	echo "${MAJOR}.${bumped_minor}.0"
}

update_patch () {
	local bumped_patch
	bumped_patch=$((PATCH + 1))
	echo "${MAJOR}.${MINOR}.${bumped_patch}"
}

# Get the url of the github repo, strip .git from the end of it
get_git_remote_url() {
	REMOTE_ORIGIN=$(git config --get remote.origin.url)
	TO_REMOVE=".git"
	GIT_REPO_URL="${REMOTE_ORIGIN//$TO_REMOVE}"
}

# Check that git status is clean
check_git_clean() {
	GIT_CLEAN=$(git status --porcelain)
	if [[ -n $GIT_CLEAN ]]
	then
		error_close "git dirty"
	fi
}

# Check currently on dev branch
check_git() {
	CURRENT_GIT_BRANCH=$(git branch --show-current)
	check_git_clean
	if [[ ! "$CURRENT_GIT_BRANCH" =~ ^dev$ ]]
	then
		error_close "not on dev branch"
	fi
}

# Ask user if current changelog is acceptable
ask_changelog_update() {
	echo "${STAR_LINE}"
	RELEASE_BODY_TEXT=$(sed '/# <a href=/Q' CHANGELOG.md)
	printf "%s" "$RELEASE_BODY_TEXT"
	printf "\n%s\n" "${STAR_LINE}"
	ask_yn "accept release body"
	if [[ "$(user_input)" =~ ^y$ ]] 
	then
		update_release_body_and_changelog "$RELEASE_BODY_TEXT"
	else
		exit
	fi
}

# Edit the release-body to include new liens from changelog
# add commit urls to changelog
# $1 RELEASE_BODY 
update_release_body_and_changelog () {
	echo -e
	DATE_SUBHEADING="### $(date +'%Y-%m-%d')\n\n"
	RELEASE_BODY_ADDITION="${DATE_SUBHEADING}$1"
	echo -e "${RELEASE_BODY_ADDITION}\n\nsee <a href='${GIT_REPO_URL}/blob/main/CHANGELOG.md'>CHANGELOG.md</a> for more details" > .github/release-body.md
	echo -e "# <a href='${GIT_REPO_URL}/releases/tag/${NEW_TAG_VERSION}'>${NEW_TAG_VERSION}</a>\n${DATE_SUBHEADING}${CHANGELOG_ADDITION}$(cat CHANGELOG.md)" > CHANGELOG.md

	# Update changelog to add links to commits [hex x40]
	# sed -i -E "s=(\s)\[([0-9a-f]{40})\](\n|\s|\,|\r)= [\2](${GIT_REPO_URL}/commit/\2),=g" ./CHANGELOG.md


	sed -i -E "s=(\s)\[([0-9a-f]{8})([0-9a-f]{32})\]= [\2](${GIT_REPO_URL}/commit/\2\3),=g" ./CHANGELOG.md

	# Update changelog to add links to closed issues
	sed -i -r -E "s=closes \[#([0-9]+)\],=[#\1](${GIT_REPO_URL}/issues/\1),=g" ./CHANGELOG.md
}

# update version in cargo.toml, to match selected current version/tag
update_cargo_toml () {
	sed -i "s|^version = .*|version = \"${NEW_TAG_VERSION:1}\"|" Cargo.toml
}

# Work out the current version, based on git tags
# create new semver version based on user input
check_tag () {
	LATEST_TAG=$(git describe --tags --abbrev=0 --always)
	echo -e "\nCurrent tag: ${PURPLE}${LATEST_TAG}${RESET}\n"
	echo -e "${YELLOW}Choose new tag version:${RESET}\n"
	if [[ $LATEST_TAG =~ ^v(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)(-((0|[1-9][0-9]*|[0-9]*[a-zA-Z-][0-9a-zA-Z-]*)(\.(0|[1-9][0-9]*|[0-9]*[a-zA-Z-][0-9a-zA-Z-]*))*))?(\+([0-9a-zA-Z-]+(\.[0-9a-zA-Z-]+)*))?$ ]]
	then
		IFS="." read -r MAJOR MINOR PATCH <<< "${LATEST_TAG:1}"
	else
		MAJOR="0"
		MINOR="0"
		PATCH="0"
	fi
	MAJOR_TAG=v$(update_major)
	MINOR_TAG=v$(update_minor)
	PATCH_TAG=v$(update_patch)
	OP_MAJOR="major___$MAJOR_TAG"
	OP_MINOR="minor___$MINOR_TAG"
	OP_PATCH="patch___$PATCH_TAG"
	OPTIONS=("$OP_MAJOR" "$OP_MINOR" "$OP_PATCH")
	select choice in "${OPTIONS[@]}"
	do
		case $choice in
			"$OP_MAJOR" )
				NEW_TAG_VERSION="$MAJOR_TAG"
				break;;
			"$OP_MINOR")
				NEW_TAG_VERSION="$MINOR_TAG"
				break;;
			"$OP_PATCH")
				NEW_TAG_VERSION="$PATCH_TAG"
				break;;
			*)
				error_close "invalid option $REPLY"
				break;;
		esac
	done
}

# ask continue, or quit
ask_continue () {
	ask_yn "continue"
	if [[ ! "$(user_input)" =~ ^y$ ]] 
	then 
		exit
	fi
}

# run all tests
cargo_test () {
	cargo test -- --test-threads=1
	ask_continue
}

# Build for linux, pi 32, pi 64, and windows
cargo_build_all() {
	cargo build --release
	cross build --target aarch64-unknown-linux-musl --release
	cross build --target arm-unknown-linux-musleabihf --release
	cross build --target x86_64-pc-windows-gnu --release
	tar -C target/arm-unknown-linux-musleabihf/release -czf ./releases/oxker_linux_armv6.tar.gz oxker
	tar -C target/aarch64-unknown-linux-musl/release -czf ./releases/oxker_linux_aarch64.tar.gz oxker
	zip -j ./releases/oxker_windows_x86_64.zip target/x86_64-pc-windows-gnu/release/oxker.exe 
	tar -C target/release -czf ./releases/oxker_linux_x86_64.tar.gz oxker
}

# Full flow to create a new release
release_flow() {
	check_git
	get_git_remote_url
	cargo_test
	cd "${CWD}" || error_close "Can't find ${CWD}"
	check_tag
	printf "\nnew tag chosen: %s\n\n" "${NEW_TAG_VERSION}"
	RELEASE_BRANCH=release-$NEW_TAG_VERSION
	echo -e
	ask_changelog_update
	git checkout -b "$RELEASE_BRANCH"
	update_cargo_toml
	cargo fmt
	git add .
	git commit -m "chore: release $NEW_TAG_VERSION"
	git checkout main
	git merge --no-ff "$RELEASE_BRANCH" -m "chore: merge ${RELEASE_BRANCH} into main"
	git tag -am "${RELEASE_BRANCH}" "$NEW_TAG_VERSION"
	echo "git tag -am \"${RELEASE_BRANCH}\" \"$NEW_TAG_VERSION\""
	git push --atomic origin main "$NEW_TAG_VERSION"
	git checkout dev
	git merge --no-ff main -m 'chore: merge main into dev'
	git branch -d "$RELEASE_BRANCH"
}


main() {
	cmd=(dialog --backtitle "Choose build option" --radiolist "choose" 14 80 16)
	options=(
		1 "fmt" off
		2 "build" off
		3 "test" off
		4 "release" off
	)
	choices=$("${cmd[@]}" "${options[@]}" 2>&1 >/dev/tty)
	exitStatus=$?
	clear
	if [ $exitStatus -ne 0 ]; then
		exit
	fi
	for choice in $choices
	do
		case $choice in
			0)
				exit
				break;;
			1)
				cargo fmt
				main
				break;;
			2)
				cargo_build_all
				main
				break;;
			3)
				npm_test
				main
				break;;
			4)
				release_flow
				break;;
		esac
	done
}

main