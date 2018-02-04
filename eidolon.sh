#!/bin/bash
#A Rofi script. If you don't want to run eidolon menu, you can just run 'rofi -show eidolon -modi eidolon:[path to this script]. This should work with combinations.
if [[ -z "$@"  ]]; then
	ls ~/.config/eidolon/games
else 
	eidolon run "$@"
fi
