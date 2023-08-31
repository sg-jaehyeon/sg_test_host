#!/bin/bash
sudo apt update
sudo apt install gtkterm can-utils -y

sudo gnome-terminal --geometry 80x27 --display=:0 --window -- /sg_test_client

