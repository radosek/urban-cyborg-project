#include <dir.h>
#include <stdio.h>
#include <string.h>
#include "datfile.h"

extern "C" {
	char **__crt0_glob_function(char *arg) {

		return 0;
	}
}


struct dat_entry *entries[2000];
int num_entries = 0;
FILE *datfd;

void init_dat(char *filename) {
	struct header hdr;

	if((datfd = fopen(filename, "wb")) == NULL) {

        	perror(filename);
                exit(1);
	}
        fwrite(&hdr, sizeof(header), 1, datfd);
}
void close_dat() {
	struct header hdr;
	hdr.magic = 0x5552424E;
        hdr.dat_start = ftell(datfd);
        hdr.num_entries = num_entries;

        fseek(datfd, 0, SEEK_SET);
        fwrite(&hdr, sizeof(header), 1, datfd);

        fseek(datfd, hdr.dat_start, SEEK_SET);

	for(int i = 0; i < num_entries; i++) {

	        fwrite(entries[i], sizeof(dat_entry), 1, datfd);
        }
        fclose(datfd);
}

void store_file(char *filename) {
	char buffer[4096];
        FILE *in;
        int n;

	if((in = fopen(filename, "rb")) == NULL) {

        	perror(filename);
                exit(1);
	}
	while((n = fread(buffer, 1, 4096, in))) {
        	fwrite(buffer, 1, n, datfd);
	}
        fclose(in);
}
void mystrlwr(char *str) {
	while(*str) {
        	if((*str >= 'A') && (*str <= 'Z'))
                	*str += 'a'-'A';
		str++;
	}
}
void store_files(char *path, char *pattern) {
	struct ffblk f;
        char buffer[128];
        char buffer2[128];
        int done;

	sprintf(buffer, "%s/*.*", path);
	done = findfirst(buffer, &f, FA_ARCH|FA_RDONLY|FA_DIREC);
	while (!done) {

                if(f.ff_attrib & FA_DIREC && f.ff_name[0]!='.') {

			sprintf(buffer2, "%s/%s", path, f.ff_name);
                        store_files(buffer2, pattern);
                }
		done = findnext(&f);
	}
	sprintf(buffer, "%s/%s", path, pattern);
	done = findfirst(buffer, &f, FA_ARCH|FA_RDONLY);
	while (!done) {

                char *tmp = f.ff_name;
		entries[num_entries] = new dat_entry;
		sprintf(entries[num_entries]->filename, "%s/%s", path, f.ff_name);
                strlwr(entries[num_entries]->filename);
		sprintf(buffer2, "%s/%s", path, f.ff_name);

                mystrlwr(buffer2);
//                strlwr(buffer2);
/*                while(*tmp) {
                	if(*tmp == 189)
                		*tmp = '™';
                      tmp++;
                }*/

                entries[num_entries]->size = f.ff_fsize;
                entries[num_entries]->offset = ftell(datfd);
                num_entries++;
		printf("FILE(%d): %s\n", num_entries+1, buffer2);

                store_file(buffer2);

		done = findnext(&f);
	}
}

void main(int argc, char **argv) {

	if(argc < 4) {

        	printf("usage: mkdat <datfile> <dirs> <pattern>");
	}
        init_dat(argv[1]);

	store_files(argv[2], argv[3]);

        close_dat();
}
