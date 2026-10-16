/* SDL2 implementation of the Allegro subset declared in allegro.h. */
#include <SDL.h>
#include <stdarg.h>
#include <stdio.h>
#include <unistd.h>
#include "allegro.h"
#include "font8x8_basic.h"

BITMAP *screen = NULL;
FONT *font = (FONT *)font8x8_basic;
PALETTE black_palette;
PALETTE desktop_palette;
volatile char key[128];
volatile int retrace_count = 0;

int num_joysticks = 0;
volatile int joy_left, joy_right, joy_up, joy_down;
volatile int joy_b1, joy_b2, joy_b3, joy_b4, joy_b5, joy_b6, joy_b7, joy_b8;

static SDL_Window *window;
static SDL_Renderer *renderer;
static SDL_Texture *texture;
static SDL_GameController *pad;
static RGB cur_pal[PAL_SIZE];
static uint32_t pal32[PAL_SIZE];
static uint32_t *argb;
static Uint64 last_present;
static int text_bg = 0;
static const char *shot_dir = getenv("URBAN_SHOTS");
static Uint64 last_shot;
static int shot_n;
static int autoplay = getenv("URBAN_AUTOPLAY") != NULL;
static Uint64 last_auto;

#define KEYBUF_SIZE 64
static int keybuf[KEYBUF_SIZE];
static int keybuf_head, keybuf_tail;

/**************************************************************************/
static int sdl_to_allegro(SDL_Scancode s) {
	switch (s) {
	case SDL_SCANCODE_ESCAPE: return KEY_ESC;
	case SDL_SCANCODE_1: return KEY_1;
	case SDL_SCANCODE_2: return KEY_2;
	case SDL_SCANCODE_3: return KEY_3;
	case SDL_SCANCODE_4: return KEY_4;
	case SDL_SCANCODE_5: return KEY_5;
	case SDL_SCANCODE_6: return KEY_6;
	case SDL_SCANCODE_7: return KEY_7;
	case SDL_SCANCODE_8: return KEY_8;
	case SDL_SCANCODE_9: return KEY_9;
	case SDL_SCANCODE_0: return KEY_0;
	case SDL_SCANCODE_MINUS: return KEY_MINUS;
	case SDL_SCANCODE_EQUALS: return KEY_EQUALS;
	case SDL_SCANCODE_BACKSPACE: return KEY_BACKSPACE;
	case SDL_SCANCODE_TAB: return KEY_TAB;
	case SDL_SCANCODE_Q: return KEY_Q;
	case SDL_SCANCODE_W: return KEY_W;
	case SDL_SCANCODE_E: return KEY_E;
	case SDL_SCANCODE_R: return KEY_R;
	case SDL_SCANCODE_T: return KEY_T;
	case SDL_SCANCODE_Y: return KEY_Y;
	case SDL_SCANCODE_U: return KEY_U;
	case SDL_SCANCODE_I: return KEY_I;
	case SDL_SCANCODE_O: return KEY_O;
	case SDL_SCANCODE_P: return KEY_P;
	case SDL_SCANCODE_LEFTBRACKET: return KEY_OPENBRACE;
	case SDL_SCANCODE_RIGHTBRACKET: return KEY_CLOSEBRACE;
	case SDL_SCANCODE_RETURN: return KEY_ENTER;
	case SDL_SCANCODE_KP_ENTER: return KEY_ENTER;
	case SDL_SCANCODE_LCTRL: return KEY_LCONTROL;
	case SDL_SCANCODE_A: return KEY_A;
	case SDL_SCANCODE_S: return KEY_S;
	case SDL_SCANCODE_D: return KEY_D;
	case SDL_SCANCODE_F: return KEY_F;
	case SDL_SCANCODE_G: return KEY_G;
	case SDL_SCANCODE_H: return KEY_H;
	case SDL_SCANCODE_J: return KEY_J;
	case SDL_SCANCODE_K: return KEY_K;
	case SDL_SCANCODE_L: return KEY_L;
	case SDL_SCANCODE_SEMICOLON: return KEY_COLON;
	case SDL_SCANCODE_APOSTROPHE: return KEY_QUOTE;
	case SDL_SCANCODE_GRAVE: return KEY_TILDE;
	case SDL_SCANCODE_LSHIFT: return KEY_LSHIFT;
	case SDL_SCANCODE_BACKSLASH: return KEY_BACKSLASH;
	case SDL_SCANCODE_Z: return KEY_Z;
	case SDL_SCANCODE_X: return KEY_X;
	case SDL_SCANCODE_C: return KEY_C;
	case SDL_SCANCODE_V: return KEY_V;
	case SDL_SCANCODE_B: return KEY_B;
	case SDL_SCANCODE_N: return KEY_N;
	case SDL_SCANCODE_M: return KEY_M;
	case SDL_SCANCODE_COMMA: return KEY_COMMA;
	case SDL_SCANCODE_PERIOD: return KEY_STOP;
	case SDL_SCANCODE_SLASH: return KEY_SLASH;
	case SDL_SCANCODE_RSHIFT: return KEY_RSHIFT;
	case SDL_SCANCODE_KP_MULTIPLY: return KEY_ASTERISK;
	case SDL_SCANCODE_LALT: return KEY_ALT;
	case SDL_SCANCODE_SPACE: return KEY_SPACE;
	case SDL_SCANCODE_CAPSLOCK: return KEY_CAPSLOCK;
	case SDL_SCANCODE_F1: return KEY_F1;
	case SDL_SCANCODE_F2: return KEY_F2;
	case SDL_SCANCODE_F3: return KEY_F3;
	case SDL_SCANCODE_F4: return KEY_F4;
	case SDL_SCANCODE_F5: return KEY_F5;
	case SDL_SCANCODE_F6: return KEY_F6;
	case SDL_SCANCODE_F7: return KEY_F7;
	case SDL_SCANCODE_F8: return KEY_F8;
	case SDL_SCANCODE_F9: return KEY_F9;
	case SDL_SCANCODE_F10: return KEY_F10;
	case SDL_SCANCODE_NUMLOCKCLEAR: return KEY_NUMLOCK;
	case SDL_SCANCODE_SCROLLLOCK: return KEY_SCRLOCK;
	case SDL_SCANCODE_HOME: return KEY_HOME;
	case SDL_SCANCODE_UP: return KEY_UP;
	case SDL_SCANCODE_PAGEUP: return KEY_PGUP;
	case SDL_SCANCODE_KP_MINUS: return KEY_MINUS_PAD;
	case SDL_SCANCODE_LEFT: return KEY_LEFT;
	case SDL_SCANCODE_KP_5: return KEY_5_PAD;
	case SDL_SCANCODE_RIGHT: return KEY_RIGHT;
	case SDL_SCANCODE_KP_PLUS: return KEY_PLUS_PAD;
	case SDL_SCANCODE_END: return KEY_END;
	case SDL_SCANCODE_DOWN: return KEY_DOWN;
	case SDL_SCANCODE_PAGEDOWN: return KEY_PGDN;
	case SDL_SCANCODE_INSERT: return KEY_INSERT;
	case SDL_SCANCODE_DELETE: return KEY_DEL;
	case SDL_SCANCODE_PRINTSCREEN: return KEY_PRTSCR;
	case SDL_SCANCODE_F11: return KEY_F11;
	case SDL_SCANCODE_F12: return KEY_F12;
	case SDL_SCANCODE_LGUI: return KEY_LWIN;
	case SDL_SCANCODE_RGUI: return KEY_RWIN;
	case SDL_SCANCODE_APPLICATION: return KEY_MENU;
	case SDL_SCANCODE_RCTRL: return KEY_RCONTROL;
	case SDL_SCANCODE_RALT: return KEY_ALTGR;
	case SDL_SCANCODE_KP_DIVIDE: return KEY_SLASH2;
	case SDL_SCANCODE_PAUSE: return KEY_PAUSE;
	default: return 0;
	}
}

static int ascii_for(SDL_Keycode k, Uint16 mod) {
	if (k >= SDLK_a && k <= SDLK_z)
		return (mod & (KMOD_SHIFT | KMOD_CAPS)) ? k - 'a' + 'A' : k;
	if (k == SDLK_RETURN || k == SDLK_KP_ENTER)
		return '\r';
	if (k == SDLK_BACKSPACE)
		return 8;
	if (k == SDLK_ESCAPE)
		return 27;
	if (k >= 32 && k < 127)
		return k;
	return 0;
}

/**************************************************************************/
static void update_pad() {
	if (!pad)
		return;
	const int dead = 12000;
	int ax = SDL_GameControllerGetAxis(pad, SDL_CONTROLLER_AXIS_LEFTX);
	int ay = SDL_GameControllerGetAxis(pad, SDL_CONTROLLER_AXIS_LEFTY);
	joy_left = ax < -dead || SDL_GameControllerGetButton(pad, SDL_CONTROLLER_BUTTON_DPAD_LEFT);
	joy_right = ax > dead || SDL_GameControllerGetButton(pad, SDL_CONTROLLER_BUTTON_DPAD_RIGHT);
	joy_up = ay < -dead || SDL_GameControllerGetButton(pad, SDL_CONTROLLER_BUTTON_DPAD_UP);
	joy_down = ay > dead || SDL_GameControllerGetButton(pad, SDL_CONTROLLER_BUTTON_DPAD_DOWN);
	joy_b1 = SDL_GameControllerGetButton(pad, SDL_CONTROLLER_BUTTON_A);
	joy_b2 = SDL_GameControllerGetButton(pad, SDL_CONTROLLER_BUTTON_B);
	joy_b3 = SDL_GameControllerGetButton(pad, SDL_CONTROLLER_BUTTON_X);
	joy_b4 = SDL_GameControllerGetButton(pad, SDL_CONTROLLER_BUTTON_Y);
	joy_b5 = SDL_GameControllerGetButton(pad, SDL_CONTROLLER_BUTTON_LEFTSHOULDER);
	joy_b6 = SDL_GameControllerGetButton(pad, SDL_CONTROLLER_BUTTON_RIGHTSHOULDER);
	joy_b7 = SDL_GameControllerGetButton(pad, SDL_CONTROLLER_BUTTON_BACK);
	joy_b8 = SDL_GameControllerGetButton(pad, SDL_CONTROLLER_BUTTON_START);
}

static void open_pad() {
	if (pad)
		return;
	for (int i = 0; i < SDL_NumJoysticks(); i++) {
		if (SDL_IsGameController(i) && (pad = SDL_GameControllerOpen(i)))
			break;
	}
	num_joysticks = pad ? 1 : 0;
}

static void toggle_fullscreen() {
	Uint32 f = SDL_GetWindowFlags(window);
	SDL_SetWindowFullscreen(window, (f & SDL_WINDOW_FULLSCREEN_DESKTOP) ? 0 : SDL_WINDOW_FULLSCREEN_DESKTOP);
}

void simulate_keypress(int k);

static void pump_events() {
	SDL_Event e;
	while (SDL_PollEvent(&e)) {
		switch (e.type) {
		case SDL_QUIT:
			exit(0);
		case SDL_KEYDOWN: {
			if (e.key.keysym.sym == SDLK_RETURN && (e.key.keysym.mod & (KMOD_ALT | KMOD_GUI))) {
				if (!e.key.repeat)
					toggle_fullscreen();
				break;
			}
			int k = sdl_to_allegro(e.key.keysym.scancode);
			if (k)
				key[k] = 1;
			int next = (keybuf_tail + 1) % KEYBUF_SIZE;
			if (k && next != keybuf_head) {
				keybuf[keybuf_tail] = (k << 8) | ascii_for(e.key.keysym.sym, e.key.keysym.mod);
				keybuf_tail = next;
			}
			break;
		}
		case SDL_KEYUP: {
			int k = sdl_to_allegro(e.key.keysym.scancode);
			if (k)
				key[k] = 0;
			break;
		}
		case SDL_CONTROLLERDEVICEADDED:
			open_pad();
			break;
		case SDL_CONTROLLERDEVICEREMOVED:
			if (pad && !SDL_GameControllerGetAttached(pad)) {
				SDL_GameControllerClose(pad);
				pad = NULL;
				open_pad();
			}
			break;
		}
	}
	update_pad();
	if (autoplay) {
		Uint64 t = SDL_GetTicks64() / 100;
		int phase = (int)(t % 30);
		if (phase == 0 && t != last_auto) {
			simulate_keypress((KEY_ENTER << 8) | '\r');
			key[KEY_ENTER] = 1;
		} else if (phase == 1) {
			key[KEY_ENTER] = 0;
		}
		key[KEY_RIGHT] = 1;
		key[KEY_LCONTROL] = (phase % 4) < 2;
		key[KEY_ALT] = phase == 10;
		last_auto = t;
	}
}

static void present(int force) {
	if (!screen || !renderer)
		return;
	Uint64 now = SDL_GetTicks64();
	if (!force && now - last_present < 8)
		return;
	last_present = now;
	int n = screen->w * screen->h;
	const unsigned char *src = screen->dat;
	for (int i = 0; i < n; i++)
		argb[i] = pal32[src[i]];
	SDL_UpdateTexture(texture, NULL, argb, screen->w * 4);
	SDL_RenderClear(renderer);
	SDL_RenderCopy(renderer, texture, NULL, NULL);
	SDL_RenderPresent(renderer);
	retrace_count++;
	if (shot_dir && now - last_shot >= 1000) {
		char path[4096];
		snprintf(path, sizeof(path), "%s/shot%04d.bmp", shot_dir, shot_n++);
		SDL_Surface *surf = SDL_CreateRGBSurfaceWithFormatFrom(argb, screen->w, screen->h, 32, screen->w * 4, SDL_PIXELFORMAT_ARGB8888);
		SDL_SaveBMP(surf, path);
		SDL_FreeSurface(surf);
		last_shot = now;
	}
}

void keyboard_update() {
	pump_events();
}

void keyboard_reset() {
	memset((void *)key, 0, sizeof(key));
}

/**************************************************************************/
__attribute__((constructor(101))) static void enter_data_dir() {
	const char *env = getenv("URBAN_DATA");
	if (env && chdir(env) == 0)
		return;
	char *base = SDL_GetBasePath();
	if (base) {
		char path[4096];
		snprintf(path, sizeof(path), "%sdata", base);
		SDL_free(base);
		if (chdir(path) == 0)
			return;
	}
}

int allegro_init() {
	if (SDL_Init(SDL_INIT_VIDEO | SDL_INIT_TIMER | SDL_INIT_GAMECONTROLLER) < 0) {
		fprintf(stderr, "SDL_Init: %s\n", SDL_GetError());
		exit(1);
	}
	atexit(allegro_exit);
	return 0;
}

void allegro_exit() {
	if (pad)
		SDL_GameControllerClose(pad);
	pad = NULL;
	if (texture)
		SDL_DestroyTexture(texture);
	if (renderer)
		SDL_DestroyRenderer(renderer);
	if (window)
		SDL_DestroyWindow(window);
	texture = NULL;
	renderer = NULL;
	window = NULL;
	SDL_Quit();
}

int install_keyboard() {
	keyboard_reset();
	return 0;
}

int install_timer() {
	return 0;
}

int install_mouse() {
	return -1;
}

int install_joystick(int type) {
	open_pad();
	return 0;
}

void remove_joystick() {
}

int poll_joystick() {
	pump_events();
	return 0;
}

/**************************************************************************/
struct TimerSlot {
	void (*proc)();
	SDL_TimerID id;
};
static TimerSlot timers[16];

static Uint32 timer_cb(Uint32 interval, void *param) {
	((TimerSlot *)param)->proc();
	return interval;
}

int install_int_ex(void (*proc)(), long speed) {
	Uint32 ms = (Uint32)(speed / (TIMERS_PER_SECOND / 1000));
	if (ms < 1)
		ms = 1;
	remove_int(proc);
	for (int i = 0; i < 16; i++) {
		if (!timers[i].proc) {
			timers[i].proc = proc;
			timers[i].id = SDL_AddTimer(ms, timer_cb, &timers[i]);
			return 0;
		}
	}
	return -1;
}

int install_int(void (*proc)(), long speed) {
	return install_int_ex(proc, MSEC_TO_TIMER(speed));
}

void remove_int(void (*proc)()) {
	for (int i = 0; i < 16; i++) {
		if (timers[i].proc == proc) {
			SDL_RemoveTimer(timers[i].id);
			timers[i].proc = NULL;
		}
	}
}

void rest(long ms) {
	present(0);
	pump_events();
	SDL_Delay((Uint32)ms);
}

const char *user_dir() {
	static char *dir;
	if (!dir)
		dir = SDL_GetPrefPath("", "urban");
	return dir ? dir : "./";
}

void wait_frame(int fps) {
	static Uint64 next;
	Uint64 freq = SDL_GetPerformanceFrequency();
	Uint64 step = freq / fps;
	Uint64 now = SDL_GetPerformanceCounter();
	if (!next || now > next + step * 4)
		next = now;
	next += step;
	while ((now = SDL_GetPerformanceCounter()) < next) {
		Uint64 left_ms = (next - now) * 1000 / freq;
		if (left_ms > 1)
			SDL_Delay((Uint32)(left_ms - 1));
	}
}

void vsync() {
	present(1);
	pump_events();
}

/**************************************************************************/
int set_gfx_mode(int card, int w, int h, int v_w, int v_h) {
	if (card == GFX_TEXT)
		return 0;
	if (!window) {
		SDL_SetHint(SDL_HINT_RENDER_SCALE_QUALITY, "0");
		window = SDL_CreateWindow("URBAN - The Cyborg Project", SDL_WINDOWPOS_CENTERED, SDL_WINDOWPOS_CENTERED,
			w * 3, h * 3, SDL_WINDOW_RESIZABLE | SDL_WINDOW_ALLOW_HIGHDPI);
		if (!window) {
			fprintf(stderr, "SDL_CreateWindow: %s\n", SDL_GetError());
			return -1;
		}
		renderer = SDL_CreateRenderer(window, -1, SDL_RENDERER_ACCELERATED);
		if (!renderer)
			renderer = SDL_CreateRenderer(window, -1, SDL_RENDERER_SOFTWARE);
		SDL_RenderSetLogicalSize(renderer, w, h);
		SDL_RenderSetIntegerScale(renderer, SDL_TRUE);
		SDL_SetRenderDrawColor(renderer, 0, 0, 0, 255);
		SDL_ShowCursor(SDL_DISABLE);
		if (getenv("URBAN_FULLSCREEN"))
			toggle_fullscreen();
	}
	if (!screen || screen->w != w || screen->h != h) {
		if (screen)
			destroy_bitmap(screen);
		if (texture)
			SDL_DestroyTexture(texture);
		free(argb);
		screen = create_bitmap(w, h);
		texture = SDL_CreateTexture(renderer, SDL_PIXELFORMAT_ARGB8888, SDL_TEXTUREACCESS_STREAMING, w, h);
		argb = (uint32_t *)malloc(w * h * 4);
	}
	clear(screen);
	keyboard_reset();
	return 0;
}

/**************************************************************************/
void set_color(int i, const RGB *p) {
	cur_pal[i] = *p;
	int r = (p->r & 63) * 255 / 63, g = (p->g & 63) * 255 / 63, b = (p->b & 63) * 255 / 63;
	pal32[i] = 0xff000000u | (r << 16) | (g << 8) | b;
}

void set_palette_range(const RGB *p, int from, int to, int vs) {
	for (int i = from; i <= to; i++)
		set_color(i, &p[i]);
	present(1);
}

void set_palette(const RGB *p) {
	set_palette_range(p, 0, PAL_SIZE - 1, 0);
}

void get_palette(RGB *p) {
	memcpy(p, cur_pal, sizeof(PALETTE));
}

void fade_interpolate(const RGB *src, const RGB *dst, RGB *out, int pos, int from, int to) {
	for (int i = from; i <= to; i++) {
		out[i].r = ((int)src[i].r * (63 - pos) + (int)dst[i].r * pos) / 64;
		out[i].g = ((int)src[i].g * (63 - pos) + (int)dst[i].g * pos) / 64;
		out[i].b = ((int)src[i].b * (63 - pos) + (int)dst[i].b * pos) / 64;
	}
}

void fade_from(const RGB *source, const RGB *dest, int speed) {
	PALETTE tmp;
	if (speed <= 0)
		speed = 1;
	for (int c = 0; c < 64; c += speed) {
		fade_interpolate(source, dest, tmp, c, 0, PAL_SIZE - 1);
		set_palette(tmp);
		pump_events();
		SDL_Delay(14);
	}
	set_palette(dest);
}

void fade_in(const RGB *p, int speed) {
	fade_from(black_palette, p, speed);
}

void fade_out(int speed) {
	PALETTE now;
	get_palette(now);
	fade_from(now, black_palette, speed);
}

/**************************************************************************/
BITMAP *create_bitmap(int w, int h) {
	BITMAP *bmp = new BITMAP;
	bmp->w = w;
	bmp->h = h;
	bmp->stride = w;
	bmp->owns_dat = 1;
	bmp->dat = new unsigned char[w * h > 0 ? w * h : 1];
	bmp->line = new unsigned char *[h > 0 ? h : 1];
	for (int i = 0; i < h; i++)
		bmp->line[i] = bmp->dat + i * w;
	set_clip(bmp, 0, 0, w - 1, h - 1);
	memset(bmp->dat, 0, w * h);
	return bmp;
}

BITMAP *create_sub_bitmap(BITMAP *parent, int x, int y, int w, int h) {
	BITMAP *bmp = new BITMAP;
	bmp->w = w;
	bmp->h = h;
	bmp->stride = parent->stride;
	bmp->owns_dat = 0;
	bmp->dat = parent->line[y] + x;
	bmp->line = new unsigned char *[h > 0 ? h : 1];
	for (int i = 0; i < h; i++)
		bmp->line[i] = parent->line[y + i] + x;
	set_clip(bmp, 0, 0, w - 1, h - 1);
	return bmp;
}

void destroy_bitmap(BITMAP *bmp) {
	if (!bmp)
		return;
	if (bmp->owns_dat)
		delete[] bmp->dat;
	delete[] bmp->line;
	delete bmp;
}

void set_clip(BITMAP *bmp, int x1, int y1, int x2, int y2) {
	if (x1 > x2 || y1 > y2) {
		x1 = 0; y1 = 0; x2 = bmp->w - 1; y2 = bmp->h - 1;
	}
	bmp->cl = x1 < 0 ? 0 : x1;
	bmp->ct = y1 < 0 ? 0 : y1;
	bmp->cr = x2 >= bmp->w ? bmp->w : x2 + 1;
	bmp->cb = y2 >= bmp->h ? bmp->h : y2 + 1;
}

/**************************************************************************/
void clear_to_color(BITMAP *bmp, int color) {
	rectfill(bmp, 0, 0, bmp->w - 1, bmp->h - 1, color);
}

void clear(BITMAP *bmp) {
	clear_to_color(bmp, 0);
}

void putpixel(BITMAP *bmp, int x, int y, int c) {
	if (x >= bmp->cl && x < bmp->cr && y >= bmp->ct && y < bmp->cb)
		bmp->line[y][x] = c;
}

int getpixel(BITMAP *bmp, int x, int y) {
	if (x < 0 || y < 0 || x >= bmp->w || y >= bmp->h)
		return -1;
	return bmp->line[y][x];
}

void rectfill(BITMAP *bmp, int x1, int y1, int x2, int y2, int c) {
	if (x2 < x1) { int t = x1; x1 = x2; x2 = t; }
	if (y2 < y1) { int t = y1; y1 = y2; y2 = t; }
	if (x1 < bmp->cl) x1 = bmp->cl;
	if (y1 < bmp->ct) y1 = bmp->ct;
	if (x2 >= bmp->cr) x2 = bmp->cr - 1;
	if (y2 >= bmp->cb) y2 = bmp->cb - 1;
	if (x1 > x2 || y1 > y2)
		return;
	for (int y = y1; y <= y2; y++)
		memset(bmp->line[y] + x1, c, x2 - x1 + 1);
}

void hline(BITMAP *bmp, int x1, int y, int x2, int c) {
	rectfill(bmp, x1, y, x2, y, c);
}

void vline(BITMAP *bmp, int x, int y1, int y2, int c) {
	rectfill(bmp, x, y1, x, y2, c);
}

void line(BITMAP *bmp, int x1, int y1, int x2, int y2, int c) {
	int dx = abs(x2 - x1), sx = x1 < x2 ? 1 : -1;
	int dy = -abs(y2 - y1), sy = y1 < y2 ? 1 : -1;
	int err = dx + dy;
	for (;;) {
		putpixel(bmp, x1, y1, c);
		if (x1 == x2 && y1 == y2)
			break;
		int e2 = 2 * err;
		if (e2 >= dy) { err += dy; x1 += sx; }
		if (e2 <= dx) { err += dx; y1 += sy; }
	}
}

void rect(BITMAP *bmp, int x1, int y1, int x2, int y2, int c) {
	hline(bmp, x1, y1, x2, c);
	hline(bmp, x1, y2, x2, c);
	vline(bmp, x1, y1, y2, c);
	vline(bmp, x2, y1, y2, c);
}

/**************************************************************************/
static int clip_blit(BITMAP *src, BITMAP *dst, int &sx, int &sy, int &dx, int &dy, int &w, int &h) {
	if (sx < 0) { w += sx; dx -= sx; sx = 0; }
	if (sy < 0) { h += sy; dy -= sy; sy = 0; }
	if (sx + w > src->w) w = src->w - sx;
	if (sy + h > src->h) h = src->h - sy;
	if (dx < dst->cl) { int d = dst->cl - dx; w -= d; sx += d; dx = dst->cl; }
	if (dy < dst->ct) { int d = dst->ct - dy; h -= d; sy += d; dy = dst->ct; }
	if (dx + w > dst->cr) w = dst->cr - dx;
	if (dy + h > dst->cb) h = dst->cb - dy;
	return w > 0 && h > 0;
}

void blit(BITMAP *src, BITMAP *dst, int sx, int sy, int dx, int dy, int w, int h) {
	if (clip_blit(src, dst, sx, sy, dx, dy, w, h)) {
		for (int y = 0; y < h; y++)
			memmove(dst->line[dy + y] + dx, src->line[sy + y] + sx, w);
	}
	if (dst == screen) {
		pump_events();
		present(1);
	}
}

void masked_blit(BITMAP *src, BITMAP *dst, int sx, int sy, int dx, int dy, int w, int h) {
	if (clip_blit(src, dst, sx, sy, dx, dy, w, h)) {
		for (int y = 0; y < h; y++) {
			const unsigned char *s = src->line[sy + y] + sx;
			unsigned char *d = dst->line[dy + y] + dx;
			for (int x = 0; x < w; x++)
				if (s[x])
					d[x] = s[x];
		}
	}
	if (dst == screen)
		present(0);
}

static void stretch_generic(BITMAP *src, BITMAP *dst, int sx, int sy, int sw, int sh,
		int dx, int dy, int dw, int dh, int masked) {
	if (dw <= 0 || dh <= 0 || sw <= 0 || sh <= 0)
		return;
	for (int y = 0; y < dh; y++) {
		int ty = dy + y;
		if (ty < dst->ct || ty >= dst->cb)
			continue;
		int syy = sy + (int)((long long)y * sh / dh);
		if (syy < 0 || syy >= src->h)
			continue;
		for (int x = 0; x < dw; x++) {
			int tx = dx + x;
			if (tx < dst->cl || tx >= dst->cr)
				continue;
			int sxx = sx + (int)((long long)x * sw / dw);
			if (sxx < 0 || sxx >= src->w)
				continue;
			unsigned char c = src->line[syy][sxx];
			if (!masked || c)
				dst->line[ty][tx] = c;
		}
	}
	if (dst == screen)
		present(0);
}

void stretch_blit(BITMAP *src, BITMAP *dst, int sx, int sy, int sw, int sh, int dx, int dy, int dw, int dh) {
	stretch_generic(src, dst, sx, sy, sw, sh, dx, dy, dw, dh, 0);
}

void stretch_sprite(BITMAP *dst, BITMAP *sprite, int x, int y, int w, int h) {
	stretch_generic(sprite, dst, 0, 0, sprite->w, sprite->h, x, y, w, h, 1);
}

void draw_sprite(BITMAP *dst, BITMAP *sprite, int x, int y) {
	masked_blit(sprite, dst, 0, 0, x, y, sprite->w, sprite->h);
}

void draw_sprite_h_flip(BITMAP *dst, BITMAP *sprite, int x, int y) {
	for (int j = 0; j < sprite->h; j++)
		for (int i = 0; i < sprite->w; i++) {
			unsigned char c = sprite->line[j][sprite->w - 1 - i];
			if (c)
				putpixel(dst, x + i, y + j, c);
		}
}

void draw_sprite_v_flip(BITMAP *dst, BITMAP *sprite, int x, int y) {
	for (int j = 0; j < sprite->h; j++)
		for (int i = 0; i < sprite->w; i++) {
			unsigned char c = sprite->line[sprite->h - 1 - j][i];
			if (c)
				putpixel(dst, x + i, y + j, c);
		}
}

/**************************************************************************/
int text_mode(int mode) {
	int old = text_bg;
	text_bg = mode;
	return old;
}

int text_length(FONT *f, const char *str) {
	return 8 * (int)strlen(str);
}

int text_height(FONT *f) {
	return 8;
}

void textout(BITMAP *bmp, FONT *f, const char *str, int x, int y, int color) {
	const char (*glyphs)[8] = font8x8_basic;
	for (; *str; str++, x += 8) {
		unsigned char ch = (unsigned char)*str;
		const char *g = glyphs[ch < 128 ? ch : '?'];
		for (int r = 0; r < 8; r++)
			for (int c = 0; c < 8; c++) {
				if (g[r] & (1 << c))
					putpixel(bmp, x + c, y + r, color);
				else if (text_bg >= 0)
					putpixel(bmp, x + c, y + r, text_bg);
			}
	}
	if (bmp == screen)
		present(0);
}

void textout_centre(BITMAP *bmp, FONT *f, const char *str, int x, int y, int color) {
	textout(bmp, f, str, x - text_length(f, str) / 2, y, color);
}

void textprintf(BITMAP *bmp, FONT *f, int x, int y, int color, const char *fmt, ...) {
	char buf[512];
	va_list ap;
	va_start(ap, fmt);
	vsnprintf(buf, sizeof(buf), fmt, ap);
	va_end(ap);
	textout(bmp, f, buf, x, y, color);
}

void textprintf_centre(BITMAP *bmp, FONT *f, int x, int y, int color, const char *fmt, ...) {
	char buf[512];
	va_list ap;
	va_start(ap, fmt);
	vsnprintf(buf, sizeof(buf), fmt, ap);
	va_end(ap);
	textout_centre(bmp, f, buf, x, y, color);
}

/**************************************************************************/
int keypressed() {
	present(0);
	pump_events();
	return keybuf_head != keybuf_tail;
}

int readkey() {
	while (!keypressed())
		SDL_Delay(5);
	int k = keybuf[keybuf_head];
	keybuf_head = (keybuf_head + 1) % KEYBUF_SIZE;
	return k;
}

void clear_keybuf() {
	pump_events();
	keybuf_head = keybuf_tail = 0;
}

void simulate_keypress(int k) {
	int next = (keybuf_tail + 1) % KEYBUF_SIZE;
	if (next != keybuf_head) {
		keybuf[keybuf_tail] = k;
		keybuf_tail = next;
	}
}
