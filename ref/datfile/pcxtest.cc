#include <stdio.h>
#include <string.h>
#include <allegro.h>

typedef unsigned char uint8;
typedef unsigned long uint32;
typedef signed long sint32;

BITMAP *tmp;

struct pcx_header {
	uint8 manufacturer		__attribute__((packed));
	uint8 version			__attribute__((packed));
	uint8 encoding			__attribute__((packed));
	uint8 bits_per_pixel		__attribute__((packed));
	uint8 xmin[2],ymin[2]		__attribute__((packed));
	uint8 xmax[2],ymax[2]		__attribute__((packed));
	uint8 hres[2]			__attribute__((packed));
	uint8 vres[2]			__attribute__((packed));

	uint8 palette16[48]		__attribute__((packed));
	uint8 reserved			__attribute__((packed));
	uint8 color_planes		__attribute__((packed));
	uint8 bytes_per_line[2]		__attribute__((packed));
	uint8 palette_type[2]		__attribute__((packed));
	uint8 filler[58]		__attribute__((packed));
};
#define get16(x) ((x[1]<<8)|x[0])
/***************************************************************************/
BITMAP * volatile load_pcx2 (char *filename, RGB *pal) {
	struct pcx_header header;
	uint32 bpp;
	sint32 i, j, c, err;
	uint32 width, height, cpl;
	uint8 *fbuf, *lptr, *nextlptr, palbuf[4];
        BITMAP *bmp;
        char found = 0;


	err = 0;

/*	for(int i = 0; i < num_entries; i++) {
		if(!strcmp(filename, entries[i]->filename)) {
                        found = 1;
                	fseek(datfd, entries[i]->offset, SEEK_SET);
                	break;
		}
        }
        if(!found) {
        	printf("\nFile not found in dat:'%s'\n", filename);
                exit(2);
        }*/
        FILE *datfd = fopen(filename, "rb");

	fread ( (char*) &header, 1, sizeof (struct pcx_header), datfd);

	width  = get16(header.xmax) - get16(header.xmin) + 1;
	height = get16(header.ymax) - get16(header.ymin) + 1;

	bpp = 1;

	cpl = header.color_planes;

//	if ((!err) && (bmp = create_bitmap(width, height)) ) {
	if ((!err) && (fbuf = (uint8 *) malloc (width * height * bpp)) ) {
//        	bmp = create_bitmap(width, height);
//		memset(bmp->dat, 0, width * height);
//		lptr = (uint8 *)bmp->dat;
		lptr = fbuf;

		for (i=0; i<(signed)height; i++) {
			nextlptr = lptr + width;
                        int x = 0;
			while ( lptr < nextlptr) {

				c = fgetc (datfd) & 0xFF;
				if ( (c & 0xC0) == 0xC0) {

					j = c & 0x3F;
					c = fgetc (datfd);
					while (j--) {
//                        			if(lptr < (nextlptr + 1))
							*lptr++ = c;
                                                        putpixel(screen, x+100, i + 100, c);
                                                        x++;
                                        }
				}
				else {
					*lptr++ = c;
                                        putpixel(screen, x+100, i + 100, c);
                                        x++;
                                }
//				blit(bmp, screen, 0, 0, 0, 0, width, height);
			}
			lptr = nextlptr;
		}
		bmp = create_bitmap(width, height);

		for (i=0; i<(signed)height; i++) {
                        memcpy(bmp->line[i], fbuf +  (i * width), width);
                }
//		memcpy(bmp->dat, fbuf, width * height);

		if ( fgetc (datfd) == 12 ) {
			for (i = 0; i < 256; i++) {

				fread (palbuf, 1, 3, datfd);
				pal[i].r = palbuf[0] >> 2;
				pal[i].g = palbuf[1] >> 2;
				pal[i].b = palbuf[2] >> 2;
			}
		}
	}
        return bmp;
}

void main() {
        allegro_init();
	install_keyboard();
        install_timer();
        PALETTE pal;

	set_gfx_mode (GFX_MODEX, 320, 240, 0, 0);

	tmp = load_bitmap ("xlife.pcx", pal);

        set_palette (pal);

	blit(tmp, screen, 0, 0, 200, 0, tmp->w, tmp->h);
//        clear(screen);

	BITMAP *xlife = load_pcx2 ("xlife.pcx", pal);

/*        set_palette (pal);*/

	blit(xlife, screen, 0, 0, 0, 0, xlife->w, xlife->h);

        while (!key[KEY_ESC]);

}
