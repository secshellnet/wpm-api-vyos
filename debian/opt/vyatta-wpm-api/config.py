#!/usr/bin/env python3
import os

from sys import exit

from vyos.config import Config
from vyos.configdict import dict_merge
from vyos.configverify import verify_vrf
from vyos.util import call
from vyos.template import render
from vyos import ConfigError
from vyos import airbag
from jinja2 import Template


airbag.enable()

config_file = r'/etc/default/wpm-api'

def get_config(config=None):
    if config:
        conf = config
    else:
        conf = Config()
    base = ['service', 'wpm-api']
    if not conf.exists(base):
        return None

    wpm_api = conf.get_config_dict(base, get_first_key=True)

    return wpm_api

def verify(wpm_api):
    if wpm_api is None:
        return None

    verify_vrf(wpm_api)
    return None

def generate(wpm_api):
    if wpm_api is None:
        if os.path.isfile(config_file):
            os.unlink(config_file)
        return None

    # merge web/listen-address with subelement web/listen-address/port
    # {'web': {'listen-address': {'0.0.0.0': {'port': '8002'}}}
    if 'listen-address' in wpm_api:
        address = list(wpm_api['listen-address'].keys())[0]
        port = wpm_api['listen-address'][address].get("port", 8002)
        wpm_api['listen-address'] = f"{address}:{port}"

    with open('/opt/vyatta-wpm-api/config.j2', 'r') as tmpl, open(config_file, 'w') as out:
        template = Template(tmpl.read()).render(data=wpm_api)
        out.write(template)

    # Reload systemd manager configuration
    call('systemctl daemon-reload')

    return None

def apply(wpm_api):
    if wpm_api is None:
        # wpm_api is removed in the commit
        call('systemctl stop wpm-api.service')
        return None

    call('systemctl restart wpm-api.service')
    return None

if __name__ == '__main__':
    try:
        c = get_config()
        verify(c)
        generate(c)
        apply(c)
    except ConfigError as e:
        print(e)
        exit(1)
