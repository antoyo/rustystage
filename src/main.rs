/*

# Files:

## List of Folders in OMGAUDIO
10F00: Contains music files (see "OMA Header file" for file structure)
A_WM
CONNECT

## Files in OMGAUDIO
00010021.DAT: File isn't modified when adding music. File is the same for NW-A608 and NW-E003 models
00GTRLST.DAT: Seems to contain explanation of tables 01TREEXX. File isn't modified when adding music
01TREE01.DAT: Gives the structure of titles in "uploaded order"
01TREE02.DAT: Gives the structure of titles in "artist order"
01TREE03.DAT: Gives the structure of titles in "album order"
01TREE04.DAT: Gives the structure of titles in "genre order"
01TREE22.DAT: Not used
01TREE2D.DAT: Gives the structure of titles in "artist-album order"
01TREEXX.DAT: (XX from 10 to 15, 2E, 2F, from 30 to 37)
02TREINF.DAT: Seems to contain explanation of tables 03GINFXX. Only few things are modified when adding music
03GINF01.DAT: Gives the name of titles in "uploaded order"
03GINF02.DAT: Gives the name of titles in "artist order"
03GINF03.DAT: Gives the name of titles in "album order"
03GINF04.DAT: Gives the name of titles in "genre order"
03GINF22.DAT: Not used
03GINF2D.DA: Gives the name of titles in "artist-album order"
03GINFXX.DAT: (XX from 10 to 15, 2E, 2F, from 30 to 37)
04CNTINF.DAT: Complete list of tracks present in the device
05CIDLST.DAT: Gives a identification of each track present on the device.
ACTIVITY.DAT:
RESERVED.DAT: Don't know which of Sonicstage or the player use it...

## Files in A_WM folder
A_WM/ARTISTLK.DAT: File isn't modified when adding music
A_WM/C2DETECT.DAT: File is modified but not too much
A_WM/EXCNTINF.DAT: File is modified. Structure is quite similar to 05CIDLST.DAT file
A_WM/EXGINF01.DAT: File isn't modified when adding music
A_WM/EXGINF02.DAT: File isn't modified when adding music
A_WM/EXTREE01.DAT: File isn't modified when adding music
A_WM/EXTREE02.DAT: File isn't modified when adding music
A_WM/MISCNIDL.DAT: File isn't modified when adding music
A_WM/MISCNMTD.DAT: File isn't modified when adding music

## Files in CONNECT folder
CONNECT/ARTSTINF.DAT: File isn't modified when adding music. Structure is quite similar to A_WM/ARTISTLK.DAT file.
CONNECT/DELCNLST.DAT: File isn't modified when adding music
CONNECT/EXCNTMTA.DAT: File is modified. Structure is quite similar to 04CNTINF.DAT without artists names

## Generations

Generations refer to Sony MP3 file manger versions
version 1.2 and earlier	version 2.0	version 2.0 + intelligent shuffle	no version compatibility	no version compatibility + covers support

Supported model
NW-E53	NW-E103	NW-A1000	NW-E002	NW-S603
NW-E55	NW-E105	NW-A1200	NW-E002F	NW-S705
NW-E73	NW-E107	NW-A3000	NW-E003	NW-S703F
NW-E75	NW-E403	NW-A608	NW-E003F
NW-E95	NW-E405		NW-E005
NW-S21	NW-E407		NW-S203F
NW-S23	NW-E503
NW-E99	NW-E505
	NW-E507

# Generalities

Here are some information that applies to all the documents.
All numbers are little endian – high then low bytes
Number begining with "0x" are hexadecimals, others are decimals
Undetermined value is zero.

Here are some explanations about vocabulary used in this document.
OmaDataBase: is The OMGAUDIO folder. An OmaDatabase contains collections of tracks and tables.
OmaTable: is a file in OMGAUDIO folder (like 01TREE01.dat)
OmaClass: is a class in a file in OMGAUDIO folder (like GPFB in 03GINF01.dat)
OmaElement: is an element of a class.

## Variables
Name: Example Comment
title_id: 0x1B2 A unique number representing the title (also the name of the corresponding file in 10F0X folders, and also the place of the element describing the title in 04CNTINF file) 
max_titled_id:  The highest title_id currently in use.
title_key: 0x0002ca63 A unique number to identify a title. Used in 03GINFxx and 04CNTINF files. It is simply the number of milliseconds of the title (same value which is given in the EA3 tag of the file itsefl)
title_id_in_TPLBlist: 0x0004 Refers to the place of the title in list of title in TPLB classes (01TREEXX files)
artist_id: 0x0006 Identifiant of an artist in the file 03GINF02. This id refers to the place of the element in this file. 
artist_key: 0x00278e4a Identifiant of an artist called ARTIST. It's the sum of all the title_key which the artist of the titles is ARTIST. Used in 03GINFxx files.
album_id:
album_key: 0x00278e4a Same as artist_key for an album.
genre_id:
genre_key: 0x00278e4a Same as artist_key for a genre.
global_key: 0x033b7c44f It's the sum of all the key (= sum of all title_key = sum of all album_key = ...)
magik_key: 0x ff ff 03 90 da 10 To be studied. Present in files : 03GINF 04CNTINF .OMA

*/

use std::fs::File;
use std::io::Read;

mod parser;

use parser::{Result, parse_table};

fn main() {
    let files = [
        "../A091-E093/OMGAUDIO/01TREE01.DAT",
        "../A091-E093/OMGAUDIO/01TREE02.DAT",
        "../A091-E093/OMGAUDIO/01TREE03.DAT",
        "../A091-E093/OMGAUDIO/01TREE04.DAT",
        "../A091-E093/OMGAUDIO/02TREINF.DAT",
        "../A091-E093/OMGAUDIO/03GINF01.DAT",
    ];
    for file in &files {
        println!("***");
        if let Err(error) = drive(file) {
            println!("Error: {}", error);
        }
    }
}

fn drive(filename: &str) -> Result<()> {
    let mut file = File::open(filename)
        .map_err(|err| err.to_string())?;
    let mut buffer =  vec![];
    file.read_to_end(&mut buffer)
        .map_err(|err| err.to_string())?;
    let table = parse_table(&buffer)?;
    println!("{}", String::from_utf8_lossy(&table.name));
    println!("{}", table.class_count);

    println!("{}", String::from_utf8_lossy(&table.class_descriptions[0].name));
    println!("{}", table.class_descriptions[0].address);
    println!("{}", table.class_descriptions[0].len);
    if table.class_descriptions.len() > 1 {
        println!("{}", String::from_utf8_lossy(&table.class_descriptions[1].name));
        println!("{}", table.class_descriptions[1].address);
        println!("{}", table.class_descriptions[1].len);
    }

    println!("{}", String::from_utf8_lossy(&table.classes[0].name));
    println!("{}", &table.classes[0].element_count);
    println!("{}", &table.classes[0].element_length);
    if table.classes.len() > 1 {
        println!("{}", String::from_utf8_lossy(&table.classes[1].name));
        println!("{}", &table.classes[1].element_count);
        println!("{}", &table.classes[1].element_length);
    }
    Ok(())
}
