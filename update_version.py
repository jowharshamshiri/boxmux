import subprocess
import re
import os
import logging
import json

logger = logging.getLogger(__name__)


def get_tag_version():
    try:
        return subprocess.check_output(["git", "describe", "--tags", "--abbrev=0"]).strip().decode()
    except subprocess.CalledProcessError:
        return "v0"


def get_commit_count_since_tag(tag_version):
    try:
        if tag_version == "v0":
            return int(subprocess.check_output(["git", "rev-list", "--count", "HEAD"]).strip().decode())
        else:
            return int(subprocess.check_output(["git", "rev-list", "--count", "{}..HEAD".format(tag_version)]).strip().decode())
    except subprocess.CalledProcessError:
        return 0


def get_total_changes():
    try:
        log_stat = subprocess.check_output(
            ["git", "log", "--pretty=tformat:", "--numstat"]).strip().decode()
        numbers = [int(s) for s in re.findall(r'\b\d+\b', log_stat)]
        return sum(numbers)
    except subprocess.CalledProcessError:
        return 0


def update_version(new_version):
    try:
        app_directory = '.'
        version_file_path = os.path.join(app_directory, "version.txt")
        logger.info("Writing version to: {}".format(
            version_file_path))  # Debugging line
        with open(version_file_path, "w") as version_file:
            version_file.write("{}\n".format(new_version))

        # Add the updated version file to the staging area
        subprocess.check_call(["git", "add", version_file_path])

    except Exception as e:
        logger.exception("Failed to update version: {}".format(e))


def update_folder_label(new_version):
    try:
        workspace_file_path = "../.vscode/machinegenesis.code-workspace"
        target_path = "../boxmux"
        label = f"boxmux v{new_version}"

        # Load the current workspace configuration
        with open(workspace_file_path, "r") as file:
            workspace_config = json.load(file)

        # Flag to track if the target path is found
        path_found = False

        # Iterate over the folders and update the name where the path matches
        for folder in workspace_config['folders']:
            if folder['path'] == target_path:
                folder['name'] = label
                path_found = True
                break  # Stop the loop once the target folder is found and updated

        # If the target path was not found, append a new folder element
        if not path_found:
            new_folder = {
                "path": target_path,
                "name": label
            }
            workspace_config['folders'].append(new_folder)

        # Write the modified configuration back to the file
        with open(workspace_file_path, "w") as file:
            json.dump(workspace_config, file, indent=2)

    except Exception as e:
        logger.exception("Failed to update folder label: {}".format(e))


if __name__ == "__main__":
    major_version = get_tag_version()
    minor_version = get_commit_count_since_tag(major_version)
    patch_version = get_total_changes()

    full_version = "{}.{}.{}".format(
        major_version[1:], minor_version, patch_version)

    update_version(full_version)
    update_folder_label(full_version)
