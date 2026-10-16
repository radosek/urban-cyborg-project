cmake_minimum_required(VERSION 3.16)
project(urban CXX)

set(CMAKE_CXX_STANDARD 98)
set(CMAKE_CXX_EXTENSIONS ON)

find_package(SDL2 REQUIRED)
find_package(PkgConfig REQUIRED)
pkg_check_modules(MIKMOD REQUIRED IMPORTED_TARGET libmikmod)
find_package(Threads REQUIRED)

set(REF ${CMAKE_SOURCE_DIR}/ref)
file(GLOB OBJECT_SOURCES ${REF}/objects/*.cc)
add_executable(urban
	port/sdl_allegro.cc
	${OBJECT_SOURCES}
	${REF}/config/config.cc
	${REF}/credits/credits.cc
	${REF}/ctrls/contrls.cc
	${REF}/ctrls/gamepad.cc
	${REF}/ctrls/keyb.cc
	${REF}/datfile/datfile.cc
	${REF}/engine/engine.cc
	${REF}/engine/game.cc
	${REF}/engine/icache.cc
	${REF}/engine/score.cc
	${REF}/font/urbfont.cc
	${REF}/gfx/flc.cc
	${REF}/highscor/highscor.cc
	${REF}/intro/intro.cc
	${REF}/map/map.cc
	${REF}/meny/meny.cc
	${REF}/outtro/outtro.cc
	${REF}/sound/scache.cc
	${REF}/sound/sound.cc
)
target_include_directories(urban PRIVATE port ${REF}/include)
target_compile_definitions(urban PRIVATE __DISTRIBUTION__)
target_compile_options(urban PRIVATE -fpermissive -Wno-write-strings -w)
target_link_libraries(urban PRIVATE SDL2::SDL2 PkgConfig::MIKMOD Threads::Threads)
add_custom_command(TARGET urban POST_BUILD
	COMMAND ${CMAKE_COMMAND} -E create_symlink ${REF}/data $<TARGET_FILE_DIR:urban>/data)
