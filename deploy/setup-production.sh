#!/bin/bash
# DEPRECATED — use the Ansible playbook at infrastructure/playbooks/provision.yml instead.
#
# This script handled compose + systemd + .env validation only. It assumed the
# host was already prepared (Docker, nginx, certbot, deploy user, sshd on 2222).
# Ansible covers the full bootstrap end-to-end and is the single source of truth
# for production provisioning and recovery.
#
# To provision or rebuild a production host:
#   cd infrastructure
#   ansible-galaxy collection install -r requirements.yml
#   ansible-playbook playbooks/provision.yml --ask-vault-pass
#
# To re-run only the application layer on an already-provisioned host:
#   ansible-playbook playbooks/resume-app.yml --ask-vault-pass

echo "ERROR: deploy/setup-production.sh is deprecated."
echo "Use: cd infrastructure && ansible-playbook playbooks/provision.yml --ask-vault-pass"
exit 1
