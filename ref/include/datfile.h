#ifndef __DATFILE_H__
#define __DATFILE_H__

#include <allegro.h>
#include <stdio.h>
#include <stdint.h>

struct dat_entry {
	char filename[120];
        int32_t size, offset;
};

struct header {
	int32_t magic;
        int32_t dat_start;
        int32_t num_entries;
};

class datfile {
	public:
	        datfile(char *filenam);
	        ~datfile();
	        BITMAP *load_pcx(char *filename, RGB *pal);
		FILE *open_file (char *filename);
		char *load_file_to_memory (char *filename);
                int GetNumEntries() {return num_entries;};
                struct dat_entry **GetEntries() {return entries;};
        private:
		struct dat_entry *entries[2000];
		FILE *datfd;
                int num_entries;
};
#endif
