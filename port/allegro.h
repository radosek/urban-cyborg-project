/* Minimal Allegro 3.11 API subset used by URBAN, implemented on SDL2. */
#ifndef URBAN_ALLEGRO_SHIM_H
#define URBAN_ALLEGRO_SHIM_H

#include <stdint.h>
#include <stdlib.h>
#include <string.h>

struct BITMAP {
	int w, h;
	int stride;
	int cl, cr, ct, cb;
	int owns_dat;
	unsigned char *dat;
	unsigned char **line;
};

struct RGB {
	unsigned char r, g, b, filler;
};

#define PAL_SIZE 256
typedef RGB PALETTE[PAL_SIZE];
typedef RGB PALLETE[PAL_SIZE];

struct FONT;

extern BITMAP *screen;
extern FONT *font;
extern PALETTE black_palette;
extern PALETTE desktop_palette;
extern volatile char key[128];
extern volatile int retrace_count;

#define SCREEN_W (screen ? screen->w : 0)
#define SCREEN_H (screen ? screen->h : 0)

#define GFX_TEXT		-1
#define GFX_AUTODETECT		0
#define GFX_VGA			1
#define GFX_MODEX		2

#define END_OF_FUNCTION(x)
#define LOCK_VARIABLE(x)
#define LOCK_FUNCTION(x)

#define TIMERS_PER_SECOND	1193181L
#define SECS_TO_TIMER(x)	((long)(x) * TIMERS_PER_SECOND)
#define MSEC_TO_TIMER(x)	((long)(x) * (TIMERS_PER_SECOND / 1000))
#define BPS_TO_TIMER(x)		(TIMERS_PER_SECOND / (long)(x))
#define BPM_TO_TIMER(x)		((60 * TIMERS_PER_SECOND) / (long)(x))

/* Allegro 3.x scancodes (kept identical so DOS-era CTRL.DAT stays valid). */
#define KEY_ESC		1
#define KEY_1		2
#define KEY_2		3
#define KEY_3		4
#define KEY_4		5
#define KEY_5		6
#define KEY_6		7
#define KEY_7		8
#define KEY_8		9
#define KEY_9		10
#define KEY_0		11
#define KEY_MINUS	12
#define KEY_EQUALS	13
#define KEY_BACKSPACE	14
#define KEY_TAB		15
#define KEY_Q		16
#define KEY_W		17
#define KEY_E		18
#define KEY_R		19
#define KEY_T		20
#define KEY_Y		21
#define KEY_U		22
#define KEY_I		23
#define KEY_O		24
#define KEY_P		25
#define KEY_OPENBRACE	26
#define KEY_CLOSEBRACE	27
#define KEY_ENTER	28
#define KEY_CONTROL	29
#define KEY_LCONTROL	29
#define KEY_A		30
#define KEY_S		31
#define KEY_D		32
#define KEY_F		33
#define KEY_G		34
#define KEY_H		35
#define KEY_J		36
#define KEY_K		37
#define KEY_L		38
#define KEY_COLON	39
#define KEY_QUOTE	40
#define KEY_TILDE	41
#define KEY_LSHIFT	42
#define KEY_BACKSLASH	43
#define KEY_Z		44
#define KEY_X		45
#define KEY_C		46
#define KEY_V		47
#define KEY_B		48
#define KEY_N		49
#define KEY_M		50
#define KEY_COMMA	51
#define KEY_STOP	52
#define KEY_SLASH	53
#define KEY_RSHIFT	54
#define KEY_ASTERISK	55
#define KEY_ALT		56
#define KEY_SPACE	57
#define KEY_CAPSLOCK	58
#define KEY_F1		59
#define KEY_F2		60
#define KEY_F3		61
#define KEY_F4		62
#define KEY_F5		63
#define KEY_F6		64
#define KEY_F7		65
#define KEY_F8		66
#define KEY_F9		67
#define KEY_F10		68
#define KEY_NUMLOCK	69
#define KEY_SCRLOCK	70
#define KEY_HOME	71
#define KEY_UP		72
#define KEY_PGUP	73
#define KEY_MINUS_PAD	74
#define KEY_LEFT	75
#define KEY_5_PAD	76
#define KEY_RIGHT	77
#define KEY_PLUS_PAD	78
#define KEY_END		79
#define KEY_DOWN	80
#define KEY_PGDN	81
#define KEY_INSERT	82
#define KEY_DEL		83
#define KEY_PRTSCR	84
#define KEY_F11		87
#define KEY_F12		88
#define KEY_LWIN	91
#define KEY_RWIN	92
#define KEY_MENU	93
#define KEY_PAD		100
#define KEY_RCONTROL	120
#define KEY_ALTGR	121
#define KEY_SLASH2	122
#define KEY_PAUSE	123

#define JOY_TYPE_AUTODETECT	-1
#define JOY_TYPE_NONE		0
#define JOY_TYPE_STANDARD	1
#define JOY_TYPE_FSPRO		2
#define JOY_TYPE_4BUTTON	3
#define JOY_TYPE_6BUTTON	4
#define JOY_TYPE_8BUTTON	5

extern int num_joysticks;
extern volatile int joy_left, joy_right, joy_up, joy_down;
extern volatile int joy_b1, joy_b2, joy_b3, joy_b4, joy_b5, joy_b6, joy_b7, joy_b8;

int allegro_init();
void allegro_exit();
int install_keyboard();
int install_timer();
int install_mouse();
int install_joystick(int type);
void remove_joystick();
int poll_joystick();

int install_int(void (*proc)(), long speed);
int install_int_ex(void (*proc)(), long speed);
void remove_int(void (*proc)());
void rest(long ms);
void vsync();

int set_gfx_mode(int card, int w, int h, int v_w, int v_h);
void set_palette(const RGB *p);
void set_palette_range(const RGB *p, int from, int to, int vsync);
void get_palette(RGB *p);
void set_color(int index, const RGB *p);
void fade_interpolate(const RGB *source, const RGB *dest, RGB *output, int pos, int from, int to);
void fade_in(const RGB *p, int speed);
void fade_out(int speed);
void fade_from(const RGB *source, const RGB *dest, int speed);

BITMAP *create_bitmap(int w, int h);
BITMAP *create_sub_bitmap(BITMAP *parent, int x, int y, int w, int h);
void destroy_bitmap(BITMAP *bmp);
void set_clip(BITMAP *bmp, int x1, int y1, int x2, int y2);

void clear(BITMAP *bmp);
void clear_to_color(BITMAP *bmp, int color);
void putpixel(BITMAP *bmp, int x, int y, int color);
int getpixel(BITMAP *bmp, int x, int y);
void hline(BITMAP *bmp, int x1, int y, int x2, int color);
void vline(BITMAP *bmp, int x, int y1, int y2, int color);
void line(BITMAP *bmp, int x1, int y1, int x2, int y2, int color);
void rect(BITMAP *bmp, int x1, int y1, int x2, int y2, int color);
void rectfill(BITMAP *bmp, int x1, int y1, int x2, int y2, int color);

void blit(BITMAP *src, BITMAP *dst, int sx, int sy, int dx, int dy, int w, int h);
void masked_blit(BITMAP *src, BITMAP *dst, int sx, int sy, int dx, int dy, int w, int h);
void stretch_blit(BITMAP *src, BITMAP *dst, int sx, int sy, int sw, int sh, int dx, int dy, int dw, int dh);
void draw_sprite(BITMAP *dst, BITMAP *sprite, int x, int y);
void draw_sprite_h_flip(BITMAP *dst, BITMAP *sprite, int x, int y);
void draw_sprite_v_flip(BITMAP *dst, BITMAP *sprite, int x, int y);
void stretch_sprite(BITMAP *dst, BITMAP *sprite, int x, int y, int w, int h);

int text_mode(int mode);
int text_length(FONT *f, const char *str);
int text_height(FONT *f);
void textout(BITMAP *bmp, FONT *f, const char *str, int x, int y, int color);
void textout_centre(BITMAP *bmp, FONT *f, const char *str, int x, int y, int color);
void textprintf(BITMAP *bmp, FONT *f, int x, int y, int color, const char *fmt, ...);
void textprintf_centre(BITMAP *bmp, FONT *f, int x, int y, int color, const char *fmt, ...);

int keypressed();
int readkey();
void clear_keybuf();
void simulate_keypress(int k);

/* Unix-port hooks the original GGI driver provided. */
void keyboard_update();
void keyboard_reset();

/* Per-user writable directory for config, saves and highscores, with trailing separator. */
const char *user_dir();

/* Sleeps until the next frame boundary at the given rate (replaces SIGALRM pacing). */
void wait_frame(int fps);

/* FLI playback lives in gfx/flc.cc. */
int play_fli(char *filename, BITMAP *bmp, int loop, int (*callback)());
int play_memory_fli(void *fli_data, BITMAP *bmp, int loop, int (*callback)());

#endif
