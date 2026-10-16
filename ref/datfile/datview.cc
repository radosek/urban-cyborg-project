#include <stdio.h>
#include <string.h>
#include <allegro.h>
#include "datfile.h"
#include "icache.h"

int main(int argc, char **argv) {
	if (argc != 2)
        	return 1;

        datfile dat(argv[1]);

        allegro_init();
	install_keyboard();
        install_timer();
	set_gfx_mode(GFX_MODEX, 320, 240, 0, 0);
        PALETTE pal;
        BITMAP *bmp;
        int running = 1, i = 0;

	while(running) {

                if((bmp = dat.load_pcx(dat.GetEntries()[i]->filename, pal))) {

                        set_palette(pal);

                        clear(screen);
			blit(bmp, screen, 0, 0,
				160 -  bmp->w / 2,  110 -  bmp->h / 2,
				bmp->w, bmp->h);

			textprintf_centre(screen, font, 160,  10, 1, "%s (%dx%d)", dat.GetEntries()[i]->filename, bmp->w, bmp->h);

                        destroy_bitmap(bmp);

			rest(50);
		}
		clear_keybuf();

                switch(readkey() >> 8) {
                	case KEY_ESC:
                        	running = 0;
                                break;

                	case KEY_UP:
                                if(i < (dat.GetNumEntries() - 1))
                                	i++;
                                break;
                	case KEY_DOWN:
                                if(i)
                                	i--;
                                break;
                }
	}
        return 0;
}

