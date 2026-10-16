/******************************************************************************

    URBAN
    Copyright (C) 1999  Thomas Nyberg and Jonas Borgstr”m

    This program is free software; you can redistribute it and/or modify
    it under the terms of the GNU General Public License as published by
    the Free Software Foundation; either version 2 of the License, or
    (at your option) any later version.

    This program is distributed in the hope that it will be useful,
    but WITHOUT ANY WARRANTY; without even the implied warranty of
    MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
    GNU General Public License for more details.

    You should have received a copy of the GNU General Public License
    along with this program; if not, write to the Free Software
    Foundation, Inc., 59 Temple Place, Suite 330, Boston, MA  02111-1307  USA

    Contact the authors:
    Snailmail:
    Thomas Nyberg					Jonas Borgstr”m
    Br„nnbergsv„gen 21b					Bondev„gen 6
    92136 Lycksele					92332 Storuman
    Sweden						Sweden

    or email:

    thomas.nyberg@usa.net				jonas_b@bitsmart.com
*****************************************************************************/
#include <allegro.h>
#include "ctrls.h"
#include "config.h"

extern Config *config;

void Gamepad_ctrl::set_key(int val, volatile int **key) {
        switch (val) {
        	case JOYSTICK_UP:
                	*key = &joy_up;
                        break;
		case JOYSTICK_DOWN:
                	*key = &joy_down;
                        break;
		case JOYSTICK_LEFT:
                	*key = &joy_left;
                        break;
		case JOYSTICK_RIGHT:
                	*key = &joy_right;
                        break;
		case JOYSTICK_B1:
                	*key = &joy_b1;
                        break;
		case JOYSTICK_B2:
                	*key = &joy_b2;
                        break;
		case JOYSTICK_B3:
                	*key = &joy_b3;
                        break;
		case JOYSTICK_B4:
                	*key = &joy_b4;
                        break;
		case JOYSTICK_B5:
                	*key = &joy_b5;
                        break;
		case JOYSTICK_B6:
                	*key = &joy_b6;
                        break;
		case JOYSTICK_B7:
                	*key = &joy_b7;
                        break;
		case JOYSTICK_B8:
                	*key = &joy_b8;
		default:
                	*key = NULL;
                	break;
        }
}

Gamepad_ctrl::Gamepad_ctrl() : Controllers() {
        int tmp;
        tmp = config->keyconf.ctrl_type;

	switch(config->keyconf.ctrl_type) {
		case CONTROLLER_GAMEPAD2:
			install_joystick(JOY_TYPE_STANDARD);
			break;

		case CONTROLLER_GAMEPAD4:
			install_joystick(JOY_TYPE_4BUTTON);
			break;

		case CONTROLLER_GAMEPAD6:
			install_joystick(JOY_TYPE_6BUTTON);
			break;

		case CONTROLLER_GAMEPAD8:
			install_joystick(JOY_TYPE_8BUTTON);
			break;

	};
        key_up = key_down = key_left = key_fire = key_jump = key_prevw = key_nextw = NULL;
        if (install_joystick(tmp)) {
        	//gick †t helvete!
	} else {
	        set_key(config->keyconf.key_up, &key_up);
        	set_key(config->keyconf.key_down, &key_down);
	        set_key(config->keyconf.key_left, &key_left);
        	set_key(config->keyconf.key_right, &key_right);
	        set_key(config->keyconf.key_fire, &key_fire);
        	set_key(config->keyconf.key_jump, &key_jump);
/*	        set_key(config->keyconf.key_prevweapon, &key_prevw);
        	set_key(config->keyconf.key_nextweapon, &key_nextw);*/
	}
}

Gamepad_ctrl::~Gamepad_ctrl() {
	remove_joystick();
}


int Gamepad_ctrl::up() {
	poll_joystick();
	if (key_up)
        	return *key_up;
	else
        	return 0;
}

int Gamepad_ctrl::down() {
	if (key_down)
        	return *key_down;
	else
        	return 0;
}

int Gamepad_ctrl::left() {
	if (key_left)
        	return *key_left;
	else
        	return 0;
}

int Gamepad_ctrl::right() {
	if (key_right)
        	return *key_right;
	else
        	return 0;
}

int Gamepad_ctrl::jump() {
	if (key_jump)
        	return *key_jump;
	else
        	return 0;
}

int Gamepad_ctrl::fire() {
	if (key_fire)
        	return *key_fire;
	else
        	return 0;
}

int Gamepad_ctrl::next_weapon() {
	static int pressed = 0;

        if (key_nextw == NULL)
        	return 0;


        if (!*key_nextw)
        	pressed = 0;

	if (*key_nextw)
        	if (pressed)
                	return 0;
		else
                	pressed = 1;

	return *key_nextw;

/*	if (key_nextw)
        	return *key_nextw;
	else
        	return 0;*/
}

int Gamepad_ctrl::prev_weapon() {
	static int pressed = 0;

        if (key_prevw == NULL)
        	return 0;


        if (!*key_prevw)
        	pressed = 0;

	if (*key_prevw)
        	if (pressed)
                	return 0;
		else
                	pressed = 1;

	return *key_prevw;
/*	if (key_prevw)
        	return *key_prevw;
	else
        	return 0;*/
}


