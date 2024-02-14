#!/bin/bash

# rust create_release v0.5.5

STAR_LINE='****************************************'
CWD=$(pwd)

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[0;33m'
PURPLE='\033[0;35m'
RESET='\033[0m'

# $1 string - error message
error_close() {
	echo -e "\n${RED}ERROR - EXITED: ${YELLOW}$1${RESET}\n"
	exit 1
}

# Check that dialog is installed
if ! [ -x "$(command -v dialog)" ]; then
	error_close "dialog is not installed"
fi

# $1 string - question to ask
ask_yn() {
	printf "%b%s? [y/N]:%b " "${GREEN}" "$1" "${RESET}"
}

# return user input
user_input() {
	read -r data
	echo "$data"
}

# ask continue, or quit
ask_continue() {
	ask_yn "continue"
	if [[ ! "$(user_input)" =~ ^y$ ]]; then
		exit
	fi
}

# semver major update
update_major() {
	local bumped_major
	bumped_major=$((MAJOR + 1))
	echo "${bumped_major}.0.0"
}

# semver minor update
update_minor() {
	local bumped_minor
	bumped_minor=$((MINOR + 1))
	MINOR=bumped_minor
	echo "${MAJOR}.${bumped_minor}.0"
}

# semver patch update
update_patch() {
	local bumped_patch
	bumped_patch=$((PATCH + 1))
	PATCH=bumped_patch
	echo "${MAJOR}.${MINOR}.${bumped_patch}"
}

# Get the url of the github repo, strip .git from the end of it
get_git_remote_url() {
	GIT_REPO_URL="$(git config --get remote.origin.url | sed 's/\.git$//')"
}

# Check that git status is clean
check_git_clean() {
	GIT_CLEAN=$(git status --porcelain)
	if [[ -n $GIT_CLEAN ]]; then
		error_close "git dirty"
	fi
}

# Check currently on dev branch
check_git() {
	CURRENT_GIT_BRANCH=$(git branch --show-current)
	check_git_clean
	if [[ ! "$CURRENT_GIT_BRANCH" =~ ^dev$ ]]; then
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
	if [[ "$(user_input)" =~ ^y$ ]]; then
		update_release_body_and_changelog "$RELEASE_BODY_TEXT"
	else
		exit
	fi
}

# Edit the release-body to include new lines from changelog
# add commit urls to changelog
# $1 RELEASE_BODY
update_release_body_and_changelog() {
	echo -e
	DATE_SUBHEADING="### $(date +'%Y-%m-%d')\n\n"
	RELEASE_BODY_ADDITION="${DATE_SUBHEADING}$1"

	# Put new changelog entries into release-body, add link to changelog
	echo -e "${RELEASE_BODY_ADDITION}\n\nsee <a href='${GIT_REPO_URL}/blob/main/CHANGELOG.md'>CHANGELOG.md</a> for more details" >.github/release-body.md

	# Add subheading with release version and date of release
	echo -e "# <a href='${GIT_REPO_URL}/releases/tag/${NEW_TAG_WITH_V}'>${NEW_TAG_WITH_V}</a>\n${DATE_SUBHEADING}${CHANGELOG_ADDITION}$(cat CHANGELOG.md)" >CHANGELOG.md

	# Update changelog to add links to commits [hex:8](url_with_full_commit)
	# "[aaaaaaaaaabbbbbbbbbbccccccccccddddddddd]" -> "[aaaaaaaa](https:/www.../commit/aaaaaaaaaabbbbbbbbbbccccccccccddddddddd)"
	sed -i -E "s=(\s)\[([0-9a-f]{8})([0-9a-f]{32})\]= [\2](${GIT_REPO_URL}/commit/\2\3)=g" CHANGELOG.md

	# Update changelog to add links to closed issues
	# "closes #1" -> "closes [#1](https:/www.../issues/1)""
	sed -i -r -E "s=closes \#([0-9]+)=closes [#\1](${GIT_REPO_URL}/issues/\1)=g" CHANGELOG.md
}

# update version in cargo.toml, to match selected current version
update_version_number_in_files() {
	sed -i "s|^version = .*|version = \"${MAJOR}.${MINOR}.${PATCH}\"|" Cargo.toml
}

# Work out the current version, based on git tags
# create new semver version based on user input
# Set MAJOR MINOR PATCH
check_tag() {
	LATEST_TAG=$(git describe --tags "$(git rev-list --tags --max-count=1)")
	echo -e "\nCurrent tag: ${PURPLE}${LATEST_TAG}${RESET}\n"
	echo -e "${YELLOW}Choose new tag version:${RESET}\n"
	if [[ $LATEST_TAG =~ ^v(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)(-((0|[1-9][0-9]*|[0-9]*[a-zA-Z-][0-9a-zA-Z-]*)(\.(0|[1-9][0-9]*|[0-9]*[a-zA-Z-][0-9a-zA-Z-]*))*))?(\+([0-9a-zA-Z-]+(\.[0-9a-zA-Z-]+)*))?$ ]]; then
		IFS="." read -r MAJOR MINOR PATCH <<<"${LATEST_TAG:1}"
	else
		MAJOR="0"
		MINOR="0"
		PATCH="0"
	fi
	OP_MAJOR="major___v$(update_major)"
	OP_MINOR="minor___v$(update_minor)"
	OP_PATCH="patch___v$(update_patch)"
	OPTIONS=("$OP_MAJOR" "$OP_MINOR" "$OP_PATCH")
	select choice in "${OPTIONS[@]}"; do
		case $choice in
		"$OP_MAJOR")
			MAJOR=$((MAJOR + 1))
			MINOR=0
			PATCH=0
			break
			;;
		"$OP_MINOR")
			MINOR=$((MINOR + 1))
			PATCH=0
			break
			;;
		"$OP_PATCH")
			PATCH=$((PATCH + 1))
			break
			;;
		*)
			error_close "invalid option $REPLY"
			;;
		esac
	done
}

# run all tests
cargo_test() {
	cargo test -- --test-threads=1
	ask_continue
}

# Simulate publishing to crates.io
cargo_publish() {
	echo -e "${PURPLE}cargo publish --dry-run${RESET}"
	cargo publish --dry-run
	ask_continue
}

# Check to see if cross is installed - if not then install
check_cross() {
	if ! [ -x "$(command -v cross)" ]; then
		echo -e "${GREEN}cargo install cross${RESET}"
		cargo install cross
	fi
}

cargo_build_x86_linux() {
	check_cross
	echo -e "${YELLOW}cross build --target x86_64-unknown-linux-musl --release${RESET}"
	cross build --target x86_64-unknown-linux-musl --release
}

cargo_build_aarch64_linux() {
	check_cross
	echo -e "${YELLOW}cross build --target aarch64-unknown-linux-musl --release${RESET}"
	cross build --target aarch64-unknown-linux-musl --release
}

cargo_build_armv6_linux() {
	check_cross
	echo -e "${YELLOW}cross build --target arm-unknown-linux-musleabihf --release${RESET}"
	cross build --target arm-unknown-linux-musleabihf --release
}

cargo_build_x86_windows() {
	check_cross
	echo -e "${YELLOW}cross build --target x86_64-pc-windows-gnu --release${RESET}"
	cross build --target x86_64-pc-windows-gnu --release
}

# Build all releases that GitHub workflow would
# This will download GB's of docker images
cargo_build_all() {
	cargo clean
	cargo_build_armv6_linux
	ask_continue
	cargo_build_aarch64_linux
	ask_continue
	cargo_build_x86_linux
	ask_continue
	cargo_build_x86_windows
	ask_continue
}

# $1 text to colourise
release_continue() {
	echo -e "\n${PURPLE}$1${RESET}"
	ask_continue
}

# Check repository for typos
check_typos() {
	echo -e "\n${PURPLE}check typos${RESET}"
	typos
	ask_continue
}

# Make sure the unused lint isn't used
check_allow_unused() {
	matches_any=$(find . -type d \( -name .git -o -name target \) -prune -o -type f -exec grep -lE '^#!\[allow\(unused\)\]$' {} +)
	matches_cargo=$(grep "^unused = \"allow\"" ./Cargo.toml)
	if [ -n "$matches_any" ]; then
		echo "\"#[allow(unused)]\" in ${matches_any}"
		ask_continue
	elif [ -n "$matches_cargo" ]; then
		echo "\"unused = \"allow\"\" in Cargo.toml"
		ask_continue
	fi
}

# Full flow to create a new release
release_flow() {
	check_allow_unused
	check_typos

	check_git
	get_git_remote_url

	cargo_test
	cargo_build_all
	cargo_publish

	cd "${CWD}" || error_close "Can't find ${CWD}"
	check_tag

	NEW_TAG_WITH_V="v${MAJOR}.${MINOR}.${PATCH}"
	printf "\nnew tag chosen: %s\n\n" "${NEW_TAG_WITH_V}"

	RELEASE_BRANCH=release-$NEW_TAG_WITH_V
	echo -e
	ask_changelog_update

	release_continue "checkout ${RELEASE_BRANCH}"
	git checkout -b "$RELEASE_BRANCH"

	release_continue "update_version_number_in_files"
	update_version_number_in_files

	echo -e "\ncargo fmt"
	cargo fmt

	echo -e "\n${PURPLE}cargo check${RESET}\n"
	cargo check

	release_continue "git add ."
	git add .

	release_continue "git commit -m \"chore: release \"${NEW_TAG_WITH_V}\""
	git commit -m "chore: release ${NEW_TAG_WITH_V}"

	release_continue "git checkout main"
	git checkout main

	echo -e "${PURPLE}git pull origin main${RESET}"
	git pull origin main

	echo -e "${PURPLE}git merge --no-ff \"${RELEASE_BRANCH}\" -m \"chore: merge ${RELEASE_BRANCH} into main\"${RESET}"
	git merge --no-ff "$RELEASE_BRANCH" -m "chore: merge ${RELEASE_BRANCH} into main"

	echo -e "\n${PURPLE}cargo check${RESET}\n"
	cargo check

	release_continue "git tag -am \"${RELEASE_BRANCH}\" \"$NEW_TAG_WITH_V\""
	git tag -am "${RELEASE_BRANCH}" "$NEW_TAG_WITH_V"

	release_continue "git push --atomic origin main \"$NEW_TAG_WITH_V\""
	git push --atomic origin main "$NEW_TAG_WITH_V"

	release_continue "git checkout dev"
	git checkout dev

	release_continue "git merge --no-ff main -m \"chore: merge main into dev\""
	git merge --no-ff main -m "chore: merge main into dev"

	release_continue "git push origin dev"
	git push origin dev

	release_continue "git branch -d \"$RELEASE_BRANCH\""
	git branch -d "$RELEASE_BRANCH"
}

build_choice() {
	cmd=(dialog --backtitle "Choose option" --radiolist "choose" 14 80 16)
	options=(
		1 "x86 musl linux" off
		2 "aarch64 musl linux" off
		3 "armv6 musl linux" off
		4 "x86 windows" off
		5 "all" off
	)
	choices=$("${cmd[@]}" "${options[@]}" 2>&1 >/dev/tty)
	exitStatus=$?
	clear
	if [ $exitStatus -ne 0 ]; then
		exit
	fi
	for choice in $choices; do
		case $choice in
		0)
			exit
			;;
		1)
			cargo_build_x86_linux
			exit
			;;
		2)
			cargo_build_aarch64_linux
			exit
			;;
		3)
			cargo_build_armv6_linux
			exit
			;;
		4)
			cargo_build_x86_windows
			exit
			;;
		5)
			cargo_build_all
			exit
			;;
		esac
	done

}

main() {
	cmd=(dialog --backtitle "Choose option" --radiolist "choose" 14 80 16)
	options=(
		1 "test" off
		2 "release" off
		3 "build" off
	)
	choices=$("${cmd[@]}" "${options[@]}" 2>&1 >/dev/tty)
	exitStatus=$?
	clear
	if [ $exitStatus -ne 0 ]; then
		exit
	fi
	for choice in $choices; do
		case $choice in
		0)
			exit
			;;
		1)
			cargo_test
			main
			break
			;;
		2)
			release_flow
			break
			;;
		3)
			build_choice
			main
			break
			;;
		esac
	done
}

main
