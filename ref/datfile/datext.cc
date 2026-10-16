#include <stdio.h>
#include <sys/stat.h>
#include <string.h>
#include <allegro.h>
#include "datfile.h"
/***************************************************************************/
void make_dirs(char *filename) {
	char buffer[80];
        int i = 0;

        while(filename[i]) {

        	if(filename[i] == '/') {

			filename[i] = 0;
	                mkdir(filename, S_IWUSR | S_IRUSR | S_IXUSR);
			filename[i] = '/';
        	}
                i++;
        }
}
/***************************************************************************/
void save_file(datfile *dat, dat_entry *entry) {
        FILE *in, *out;

        make_dirs(entry->filename);

        if((in = dat->open_file(entry->filename)) == NULL) {

                printf("Couldn't open '%s' in datfile\n", entry->filename);
                exit(1);
	}

        if((out = fopen(entry->filename, "wb")) == NULL) {

                printf("Create '%s'\n", entry->filename);
                exit(2);
	}
        char *buf = new char[entry->size];
        fread(buf, 1, entry->size, in);
        fwrite(buf, 1, entry->size, out);

        fclose(out);
}
/***************************************************************************/
int main(int argc, char **argv) {
	if (argc != 2)
        	return 1;

        datfile dat(argv[1]);

        for(int i = 0; i < dat.GetNumEntries(); i++) {

		printf("Extracting: %s\n", dat.GetEntries()[i]->filename);
                save_file(&dat, dat.GetEntries()[i]);
	}

}
/***************************************************************************/

