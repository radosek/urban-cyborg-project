#include <stdio.h>
#include <string.h>
#include <allegro.h>
#include "datfile.h"

int main(int argc, char **argv) {
	if (argc != 2)
        	return 1;

        datfile dat(argv[1]);

        for(int i = 0; i < dat.GetNumEntries(); i++) {

		printf("%s\n", dat.GetEntries()[i]->filename);
	}

}

