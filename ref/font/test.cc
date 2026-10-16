#include <allegro.h>
#include "icache.h"
#include "urbfont.h"

ImageCache icache;

void main() {
	UrbanFont *ufo;
	UrbanFont *ufo2;

        allegro_init();
	install_keyboard();
        install_timer();
	set_gfx_mode(GFX_MODEX, 320, 240, 0, 0);

//        ufo = new UrbanFont("dafont2.pcx");
        ufo = new UrbanFont(LARGE_FONT);
//        ufo->SetScale(50);

        ufo->print("HELLO WORLD\nURBAN RULEZ\nMUD SUGER", 0, 0, screen);
        ufo2 = new UrbanFont(SMALL_FONT);
//        ufo2->SetScale(150);

		char credits[] = "Lead Programmers
Jonas Borgstr”m and Thomas Nyberg

Graphics
Dick Adolfsson

Story
Petter Holmberg and Marcus Boman

Sound effects
Emil Eriksson and Rickard Bergmark

Special Thanks to
Per-Jonny K„ck

Thanks to
Henrik Johansson";

        ufo2->print(credits, 50, 100, screen);
//        ufo2->print("HELLO WORLD\nURBAN RULEZ\nMUD SUGER", 50, 100, screen);
//        ufo->print("Hello World!\nDetta „r en ny rad!!\nOch en till!", 0, 0, screen);

        while (!key[KEY_ESC]);
        set_gfx_mode(GFX_TEXT, 80, 25, 0, 0);
}
