use std::result;

pub type Result<T> = result::Result<T, String>;

/*

# Table

This sheet contains explanations about the global structure of a table

## A table is : a table header
	a class descriptions
	a list of classes

## Table header :
The header of a table contains the name of the table and the number of classes :
Adress	Length	Value	Comment
0	4 bytes	Table name (like TREE, GINF,...)
4	4 bytes	0x1100	constant
8	1 byte	Number of classes

## Classes description
There is one class description per class
Adress	Length	Value	Comment
16 for the 1st  class 32 for the 2nd class	4 bytes	Class name (like GPLB, TPLB,...)
20 or 36	4 bytes	Start adress in the file in hexa	This number must end with 0 (in hexa).
24 or 40	4 bytes	Length of the class in hexa	This number must end with 0 (in hexa).

## Class
A class is:	a class header
	a list of elements

## Class header
Each class begins with an header that contains the name of the class, the number of element 
Adress	Length	Value	Comment
X (the first adress of the class)	4 bytes	Class name (like GPLB, TPLB,...)
X + 4	2 bytes	Number of elements (in hexa)
X + 6	2 bytes	Length of one element (in hexa)

## List of elements
This can't be described here, it depends on each class.

*/

// TODO: remove fields only useful for parsing?

pub struct Class {
    pub name: Vec<u8>,
    pub element_count: u16,
    pub element_length: u16,
    pub kind: ClassKind,
}

pub struct ClassDescription {
    pub name: Vec<u8>,
    pub address: u32,
    pub len: u32,
}

pub enum ClassKind {
    Gplb(Vec<GplbElement>),
    Tplb(Vec<TplbElement>),
}

pub struct GplbElement {
    pub id: u16,
    pub association: u16,
    pub title_id: u16,
}

pub struct Table {
    pub classes: Vec<Class>,
    pub class_descriptions: Vec<ClassDescription>,
    pub class_count: u8,
    pub name: Vec<u8>,
}

pub struct TplbElement {
    pub title_id: u16,
}

pub fn parse_table(buffer: &[u8]) -> Result<Table> {
    let mut parser = Parser::new(buffer);
    parser.table()
}

struct Parser<'a> {
    buffer: &'a [u8],
    index: usize,
}

impl<'a> Parser<'a> {
    fn new(buffer: &'a [u8]) -> Self {
        Self {
            buffer,
            index: 0,
        }
    }

    fn klass(&mut self, class_description: &ClassDescription) -> Result<Class> {
        let current_index = self.index;
        self.take(class_description.address as usize - current_index)?;
        let name = self.take(4)?.to_vec();
        let element_count = self.u16()?;
        let element_length = self.u16()?;
        let kind = self.kind(&name, element_count)?;
        Ok(Class {
            name,
            element_count,
            element_length,
            kind,
        })
    }

    fn class_description(&mut self) -> Result<ClassDescription> {
        let name = self.take(4)?.to_vec();
        let address = self.u32()?;
        let len = self.u32()?;
        self.take(4)?;
        Ok(ClassDescription {
            name,
            address,
            len,
        })
    }

    fn kind(&mut self, name: &[u8], element_count: u16) -> Result<ClassKind> {
        match name {
            b"GPLB" => {
                let mut elements = vec![];
                for _ in 0..element_count {
                    let id = self.u16()?;
                    let association = self.u16()?;
                    let title_id = self.u16()?;
                    self.take(2)?;
                    elements.push(GplbElement {
                        id,
                        association,
                        title_id,
                    });
                }
                Ok(ClassKind::Gplb(elements))
            },
            b"TPLB" => {
                let mut elements = vec![];
                for _ in 0..element_count {
                    let title_id = self.u16()?;
                    elements.push(TplbElement {
                        title_id,
                    });
                }
                Ok(ClassKind::Tplb(elements))
            },
            _ => Err(format!("Unknown class kind {}", String::from_utf8_lossy(name))),
        }
    }

    fn table(&mut self) -> Result<Table> {
        let name = self.take(4)?.to_vec();
        self.eat_u32(0x01010000)?;
        let class_count = self.u8()?;
        let current_index = self.index;
        self.take(16 - current_index)?;
        let mut classes = vec![];
        let mut class_descriptions = vec![];
        for _ in 0..class_count {
            class_descriptions.push(self.class_description()?);
        }
        for class_description in &class_descriptions {
            classes.push(self.klass(&class_description)?);
        }
        Ok(Table {
            classes,
            class_descriptions,
            class_count,
            name,
        })
    }

    fn eat_u32(&mut self, num: u32) -> Result<()> {
        let bytes = self.take(4)?;
        let actual = (bytes[0] as u32) << 24 | (bytes[1] as u32) << 16 | (bytes[2] as u32) << 8 | bytes[3] as u32;
        if num == actual {
            Ok(())
        }
        else {
            Err(format!("Expected number {:x}, actual number {:x}", num, actual))
        }
    }

    fn take(&mut self, len: usize) -> Result<&[u8]> {
        if self.buffer.len() - self.index >= len {
            let index = self.index;
            self.index += len;
            Ok(&self.buffer[index..index + len])
        }
        else {
            Err(format!("Trying to take a slice of len {} in the buffer of len {}", len, self.buffer.len()))
        }
    }

    fn u16(&mut self) -> Result<u16> {
        let bytes = self.take(4)?;
        Ok((bytes[2] as u16) << 8 | bytes[3] as u16)
    }

    fn u32(&mut self) -> Result<u32> {
        let bytes = self.take(4)?;
        Ok((bytes[0] as u32) << 24 | (bytes[1] as u32) << 16 | (bytes[2] as u32) << 8 | bytes[3] as u32)
    }

    fn u8(&mut self) -> Result<u8> {
        let bytes = self.take(1)?;
        Ok(bytes[0])
    }
}

/*

# 00GTRLST

## Table header
Table name	"GTLT"
Number of classes	2

## Class 1
Class name	"SYSB"
Start adress	0x000030
Length	0x000070
Number of elements	1
Length of one element	50
Class header X + 8	0xd0 00 00 00 00 00 00 00

### Elements
There is only one element and it's full of zeros. (There are 0x10 octets full of zeros after the element)

### Explanation
This class don't have to be change.

## Class 2
Class name	"GTLB"
Start adress	0x0000a0
Length	(Number_of_elements * Length_of_one_element) + 0x10
Number of elements	0x2d
Length of one element	50
Class header X + 8	0x00 00 00 06 04 00 00 00

### Elements
Element 01:
Adress	Name	Length	Value	Comment
0xb0	fileRef	2 bytes	0x01	Refers to file 01TREE01 and 03GINF01
0xb2	unknown1	2 bytes	0x01
0xc0	numberOfTag	4 bytes	0x00 01
0xc4	tag1	4 bytes	0x00 00 00 00
### Element 02:
Adress	Name	Length	Value	Comment
0x100	fileRef	2 bytes	0x02	Refers to file 01TREE02 and 03GINF02
0x102	unknown1	2 bytes	0x03
0x110	numberOfTag	4 bytes	0x00 01	1 seems to mean that 1 tag ref is following
0x114	tag1	4 bytes	"TPE1"	"TPE1" = "Artist" in ID3 tags
### Element 03:
Adress	Name	Length	Value	Comment
0x150	fileRef	2 bytes	0x03	Refers to file 01TREE03 and 03GINF03
0x152	unknown1	2 bytes	0x03
0x160	numberOfTag	2 bytes	0x00 01	1 seems to mean that 1 tag ref is following
0x164	tag1	4 bytes	"TALB"	"TALB" = "Album" in ID3 tags
### Element 04:
Adress	Name	Length	Value	Comment
0x1a0	fileRef	2 bytes	0x04	Refers to file 01TREE04 and 03GINF04
0x1a2	unknown1	2 bytes	0x03
0x1b0	numberOfTag	2 bytes	0x00 01	1 seems to mean that 1 tag ref is following
0x1b4	tag1	4 bytes	"TCON"	"TCON" = "Genre" in ID3 tags
### Element 05:
Adress	Name	Length	Value	Comment
0x1f0	fileRef	2 bytes	0x22	Refers to file 01TREE22 and 03GINF22
0x1f2	unknown1	2 bytes	0x02
0x200	numberOfTag	2 bytes	0x00 00
0x204	tag1	4 bytes	0x00 00 00 00
### Element 06:
Adress	Name	Length	Value	Comment
0x240	fileRef	2 bytes	0x2d	Refers to file 01TREE2d and 03GINF2d
0x242	unknown1	2 bytes	0x03
0x250	numberOfTag	2 bytes	0x00 02 	2 seems to mean that 2 tag ref are following
0x254	tag1	4 bytes	"TPE1"
0x258	tag1	4 bytes	"TALB"
0X270	unknown2	0x0c bytes	"TRNOTTCCTTCC"
### Element 07 to 2d:
Adress	Length	Value	Comment
X (the first adress of the element)	2 bytes		This value goes from 05 to 2c, avoiding 22

## Explanation
This class seems to explain the content of the files in OMGAUDIO.
This class don't have to be change.

*/

/*

# 01TREE01

## Table header
Table name	"TREE"
Number of classes	2

## Class 1
Class name	"GPLB"
Start adress	0x000030
Length	0x004010	Class length is constant
Number of elements	Number of elements in 03GINF01 file	Could be the number of albums, but unused albums are listed here, and albums uploaded in different music update are listed several times
Length of one element	0x8
Class header X + 0a	Number of elements in 03GINF01 file	This information is repeated, I don't know why...

### Elements
Element content
Adress	Length	Value	Comment
X (the first adress of the element)	2 bytes	item_id_in_03GINF01
X + 2	2 bytes	0x0100or 0x0000	0x0100 when the album is associated to titles, 0x0000 when the album is present in description file (03GING01) but no title of this album exist in the player.
X + 4	2 bytes	title_id_in_TPLBlist
X + 6	2 bytes	0x0000	Constant

### Explanation
This class can't be explain without using the second class of this file and files 03GINF01 and 04CNTINF.
To make it easy, I take an example (at the right).
You can see a list of titles, the corresponding 03GINF01 and 04CNTINF files and what should be GPLB and TPLB classes.
I suppose that a title in 03GINF01 isn't used (also present in 04CNTINF).
The GPLB class gives the item_id_in_03GINF01 in "uploaded" order and gives the element in TPLB list of the first title of the corresponding album (see example to understand).
The "uploaded" order means that if several tracks of the same album are loaded in several times, the album will appears several times in this file.
When there are albums listed in 03GINF01 which are not used, there are put at the end of GPLB list with constant 0x0000 instead of 0x0100.
So in GPLB, there is the list of used albums in "upladed" order and then the list of unused albums (I suppose also in alphabetical order, but this is certainly not important)
Exemple of title :
(Artist – Album – Track number – Track title)
Archive – Noise – 02 – Fuck U
Archive – Noise – 03 – Waste
Archive – Take My Head – 01 – You make me feel
Archive – Take My Head – 02 – The way you love me
Archive – Take My Head – 07 – Cloud in the sky
Radiohead – Amnesiac – 02 – Pyramid song
Radiohead – Amnesiac – 05 – I migth be wrong
Radiohead – Amnesiac – 06 – Knives out
Radiohead – Amnesiac – 09 – Hunting Bears
04CNTINF:
Element01: Radiohead – Amnesiac – 05 – I migth be wrong
Element02: Radiohead – Amnesiac – 02 – Pyramid song
Element03: Archive – Take My Head – 01 – You make me feel
Element04: Archive – Take My Head – 02 – The way you love me
Element05: Archive – Take My Head – 07 – Cloud in the sky
Element06: unused
Element07: Archive – Noise – 02 – Fuck U
Element08: Archive – Noise – 03 – Waste
Element09: Radiohead – Amnesiac – 06 – Knives out
Element0a: Radiohead – Amnesiac – 09 – Hunting Bears
03GINF01:
Element1: Amnesiac
Element2: Take My Hand
Element3: Offspring (unused)
Element4: Noise
Element5 : Amnesiac
GPLB:
Element1: 01 0x0100 01
Element2: 02 0x0100 03
Element3: 04 0x0100 06
Element4: 05 0x0100 08
Element5: 03 0x0000

Can be translated to :
- The first uploaded album is the element 01 of 03GINF01 (=Amnesiac) and its first title in TPLB list is the element 01
- The second album is the element 02 of 03GINF01 (=Take My Head) and its first title in TPLB list is the element 03
- The third album is the element 04 of 03GINF01 (=Noise) and its first title in TPLB list is the element 06
- The fourth album is the element 05 of 03GINF01 (=Amnesiac) and its first title in TPLB list is the element 08
- The fifth album is the element 03 of 03GINF01 (=Offspring) but it is unused.
TPLB:
Element01: 01 (= Radiohead – Amnesiac – 05 – I migth be wrong)
Element02: 02 (= Radiohead – Amnesiac – 02 – Pyramid song)
Element03: 03 (= Archive – Take My Head – 01 – You make me feel)
Element04: 04 (= Archive – Take My Head – 02 – The way you love me)
Element05: 05 (= Archive – Take My Head – 07 – Cloud in the sky)
Element06: 07 (= Archive – Noise – 02 – Fuck U)
Element07: 08 (= Archive – Noise – 03 – Waste)
Element08: 09 (= Radiohead – Amnesiac – 06 – Knives out)
Element09: 0a (= Radiohead – Amnesiac – 09 – Hunting Bears)
Element0a: 06 (= unused)
Here, titles are listed in "Uploaded order".
The number given is title_id corresponding to the place of the title in 04CNTINF.

### Adding titles
This is explained in "Adding titles" section of TPLB class.

### Deleting titles
This is explained in "Deleting titles" section of TPLB class.

## Class 2
Class name	"TPLB"
Start adress	0x004040	Start adress is constant as previous class length is constant
Length	(Number_of_elements * Length_of_one_element) + 0x10
Number of elements	Number of used titles in the player
Length of one element	0x2
Class header X + 0a	Number of used titles in the player	This information is repeated, I don't know why...

### Elements
Element content
Adress	Length	Value	Comment
X (the first adress of the element)	2 bytes	title_id

### Explanation
As GPLB class, this class can't be explained without 03GINF01 and 04CNTINF files and GPLB.
See GPLB example as example.
This class gives all the title_id in "uploaded order" whatever the name of the album and the track number.
In this class, only used track are listed.
(even if I saw example where unused track are listed, but I don't think it's compulsory)

### Adding titles
When you add a title, you ony have to add information at the end of each class. Added information (i.e. added titles) must be sort in "the upload order". 

### Deleting titles
Deleting titles is not the same. You must rebuild entirely this file (not necessarly GPLB class but always TPLB list). First, you sort used titles in "the good order" = "uploaded order" and you write the title_id according to this sort in TPLB class. Then, to fill in GPLB class, you list all the used items of 03GING01 in "uploaded order" (i.e. the order of appearance) and for each (in this order) you write the item_id, the 0x0100 constant and the title_id_in_TPLBlist. Then you write all the unused items with the 0x0000 constant.

*/

/*

# 01TREE02


## Table header
Table name	"TREE"
Number of classes	2 bytes

## Class 1
Class name	"GPLB"
Start adress	0x000030
Length	0x004010	Class length is constant
Number of elements	Number of elements in 03GINF02 file (used and unused artists)	Could be the number of artists, but unused artists are listed here
Length of one element	0x8
Class header X + 0a	Number of USED artitsts

### Elements
Element content
Adress	Length	Value	Comment
X (the first adress of the element)	2 bytes	artist_id
X + 2	2 bytes	0x0100or 0x0000	0x0100 when the artist is associated to titles, 0x0000 when the artist is present in description file (03GING02) but no title of this artist exist in the player.
X + 4	2 bytes	title_id_in_TPLBlist
X + 6	2 bytes	0x0000	Constant

### Explanation
This class can't be explain without using the second class of this file and files 03GINF02 and 04CNTINF.
To make it easy, I take an example (at the right).
You can see a list of titles, the corresponding 03GINF02 and 04CNTINF files and what should be GPLB and TPLB classes.
I suppose that an artist in 03GINF02 isn't used and a title in 04CNTINF isn't used.
The GPLB class gives the artist_id in alphabetical order and gives the element in TPLB list of the first title of the corresponding artist (see example to understand).
When there are artists listed in 03GINF02 which are not used, there are put at the end of GPLB list with constant 0x0000 instead of 0x0100.
So in GPLB, there is the list of used artist in alphabetical order and then the list of unused artist (I suppose also in alphabetical order, but this is certainly not important)
Exemple of title :
(Artist – Album – Track number – Track title)
Archive – Noise – 02 – Fuck U
Archive – Noise – 03 – Waste
Archive – Take My Head – 01 – You make me feel
Archive – Take My Head – 02 – The way you love me
Archive – Take My Head – 07 – Cloud in the sky
Radiohead – Amnesiac – 02 – Pyramid song
Radiohead – Amnesiac – 05 – I migth be wrong
Radiohead – Amnesiac – 06 – Knives out
Radiohead – Amnesiac – 09 – Hunting Bears
04CNTINF:
Element01: Radiohead – Amnesiac – 02 – Pyramid song
Element02: Radiohead – Amnesiac – 05 – I migth be wrong
Element03: Radiohead – Amnesiac – 06 – Knives out
Element04: Radiohead – Amnesiac – 09 – Hunting Bears
Element05: Archive – Take My Head – 01 – You make me feel
Element06: Archive – Take My Head – 02 – The way you love me
Element07: Archive – Take My Head – 07 – Cloud in the sky
Element08: unused
Element09: Archive – Noise – 02 – Fuck U
Element0a: Archive – Noise – 03 – Waste
03GINF02:
Element1:
Radiohead
Element2: Archive
Element3: Offspring (unused)
GPLB:
Element1: 02 0x0100 01
Element2: 01 0x0100 06
Element3: 03 0x0000 00
Can be translated to :
- The first artist is the element 02 of 03GINF02 (=Archive) and its first title in TPLB list is the element 01
- The second artist is the element 01 of 03GINF02 (=Radiohead) and its first title in TPLB list is the element 06
- The third artist is the element 03 of 03GINF02 (=Offspring) but it is unused.
TPLB:
Element01: 07 (= Archive – Cloud in the sky [Take My Head – 07]  )
Element02: 09 (= Archive – Fuck U [Noise – 02])
Element03: 06 (= Archive – The way you love me [Take My Head – 02])
Element04: 0a (= Archive – Waste [Noise – 03])
Element05: 05 (= Archive – You make me feel [Take My Head – 01])
Element06: 04 (= Radiohead – Hunting Bears [Amnesiac – 09])
Element07: 02 (= Radiohead – I migth be wrong [Amnesiac – 05])
Element08: 03 (= Radiohead – Knives out [Amnesiac – 06])
Element09: 01 (= Radiohead – Pyramid song [Amnesiac – 02])
Here, titles are listed in "Artist alphabetical order – Title name alphabetical order".
The number given is title_id corresponding to the place of the title in 04CNTINF.

### Adding titles
This is explained in "Adding titles" section of TPLB class.

### Deleting titles
This is explained in "Deleting titles" section of TPLB class.

## Class 2
Class name	"TPLB"
Start adress	0x004040	Start adress is constant as previous class length is constant
Length	(Number_of_elements * Length_of_one_element) + 0x10
Number of elements	Number of used titles in the player
Length of one element	0x2
Class header X + 0a	Number of used titles in the player	This information is repeated, I don't know why...

### Elements
Element content
Adress	Length	Value	Comment
X (the first adress of the element)	2 bytes	title_id

### Explanation
As GPLB class, this class can't be explained without 03GINF02 and 04CNTINF files and GPLB.
See GPLB example as example.
This class gives all the title_id in "artist alphabetical order – title name alphabetical order" whatever the name of the album and the track  number.
In this class, only used track are listed.

### Adding titles
When you add a title, you have to rebuild entirely this file (not necessarly GPLB class but always TPLB list). First, you sort titles in "the good order" = "artist alphabetical order – title name alphabetical order" and you write the title_id according to this sort in TPLB class. Then, to fill in GPLB class, you list all the used artists (in alphabetical order) and for each (in this order) you write the artist_id, the 0x0100 constant and the title_id_in_TPLBlist. Then you write all the unused artists with the 0x0000 constant.

### Deleting titles
To delete a title, you have to make the exactly same thing as "Adding titles".

*/

/*

# 01TREE03

## Table header
Table name	"TREE"
Number of classes	2 bytes

## Class 1
Class name	"GPLB"
Start adress	0x000030
Length	0x004010	Class length is constant
Number of elements	Number of elements in 03GINF03 file	Could be the number of albums, but unused albums are listed here
Length of one element	0x8
Class header X + 0a	Number of elements in 03GINF03 file	This information is repeated, I don't know why...

### Elements
Element content
Adress	Length	Value	Comment
X (the first adress of the element)	2 bytes	album_id
X + 2	2 bytes	0x0100or 0x0000	0x0100 when the album is associated to titles, 0x0000 when the album is present in description file (03GING03) but no title of this album exist in the player.
X + 4	2 bytes	title_id_in_TPLBlist
X + 6	2 bytes	0x0000	Constant

### Explanation
This class can't be explain without using the second class of this file and files 03GINF03 and 04CNTINF.
To make it easy, I take an example (at the right).
You can see a list of titles, the corresponding 03GINF03 and 04CNTINF files and what should be GPLB and TPLB classes.
I suppose that an album in 03GINF03 isn't used and a title in 04CNTINF isn't used.
The GPLB class gives the album_id in alphabetical order and gives the element in TPLB list of the first title of the corresponding album (see example to understand).
When there are albums listed in 03GINF03 which are not used, there are put at the end of GPLB list with constant 0x0000 instead of 0x0100.
So in GPLB, there is the list of used albums in alphabetical order and then the list of unused artist (I suppose also in alphabetical order, but this is certainly not important)
Exemple of title :
(Artist – Album – Track number – Track title)
Archive – Noise – 02 – Fuck U
Archive – Noise – 03 – Waste
Archive – Take My Head – 01 – You make me feel
Archive – Take My Head – 02 – The way you love me
Archive – Take My Head – 07 – Cloud in the sky
Radiohead – Amnesiac – 02 – Pyramid song
Radiohead – Amnesiac – 05 – I migth be wrong
Radiohead – Amnesiac – 06 – Knives out
Radiohead – Amnesiac – 09 – Hunting Bears
04CNTINF:
Element01: Radiohead – Amnesiac – 02 – Pyramid song
Element02: Radiohead – Amnesiac – 05 – I migth be wrong
Element03: Radiohead – Amnesiac – 06 – Knives out
Element04: Radiohead – Amnesiac – 09 – Hunting Bears
Element05: Archive – Take My Head – 01 – You make me feel
Element06: Archive – Take My Head – 02 – The way you love me
Element07: Archive – Take My Head – 07 – Cloud in the sky
Element08: unused
Element09: Archive – Noise – 02 – Fuck U
Element0a: Archive – Noise – 03 – Waste
03GINF03:
Element1: Take My Head
Element2: Amnesiac
Element3: Noise
Element4: Americana (unused)
GPLB:
Element1: 02 0x0100 01
Element2: 03 0x0100 05
Element3: 01 0x0100 07
Element4: 04 0x0000 00
Can be translated to :
- The first album is the element 02 of 03GINF03 (=Amnesiac) and its first title in TPLB list is the element 01
- The second album is the element 03 of 03GINF03 (=Noise) and its first title in TPLB list is the element 05
- The third album is the element 01 of 03GINF03 (=Take My Head) and its first title in TPLB list is the element 07.
- The fourth album is the element 04 of 03GINF03 (=Americana) but it is unused.
TPLB:
Element01: 01 (= Amnesiac – 02 [Radiohead – Pyramid song])
Element02: 02 (= Amnesiac – 05 [Radiohead – I migth be wrong])
Element03: 03 (= Amnesiac – 06 [Radiohead – Knives out])
Element04: 04 (= Amnesiac – 09 [Radiohead – Hunting Bears])
Element05: 09 (= Noise – 02 [Archive – Fuck U])
Element06: 0a (= Noise – 03 [Archive – Waste])
Element07: 05 (= Take My Head – 01 [Archive – You make me feel])
Element08: 06 (= Take My Head – 02 [Archive – The way you love me])
Element09: 07 (= Take My Head – 07 [Archive – Cloud in the sky]  )
Here, titles are listed in "Album alphabetical order – Track number order".
The number given is title_id corresponding to the place of the title in 04CNTINF.

### Adding titles
This is explained in "Adding titles" section of TPLB class.

### Deleting titles
This is explained in "Deleting titles" section of TPLB class.

## Class 2
Class name	"TPLB"
Start adress	0x004040	Start adress is constant as previous class length is constant
Length	(Number_of_elements * Length_of_one_element) + 0x10
Number of elements	Number of used titles in the player
Length of one element	0x2
Class header X + 0a	Number of used titles in the player	This information is repeated, I don't know why...

### Elements
Element content
Adress	Length	Value	Comment
X (the first adress of the element)	2 bytes	title_id

### Explanation
As GPLB class, this class can't be explained without 03GINF03 and 04CNTINF files and GPLB.
See GPLB example as example.
This class gives all the title_id in "album alphabetical order – track number order" whatever the name of the artist and the name of the title.
In this class, only used track are listed.

### Adding titles
When you add a title, you have to rebuild entirely this file (not necessarly GPLB class but always TPLB list). First, you sort titles in "the good order" = "album alphabetical order – track number order" and you write the title_id according to this sort in TPLB class. Then, to fill in GPLB class, you list all the used albums (in alphabetical order) and for each (in this order) you write the album_id, the 0x0100 constant and the title_id_in_TPLBlist. Then you write all the unused artists with the 0x0000 constant.

### Deleting titles
To delete a title, you have to make the exactly same thing as "Adding titles".

*/

/*

# 01TREE04

## Table header
Table name	"TREE"
Number of classes	2 bytes

## Class 1
Class name	"GPLB"
Start adress	0x000030
Length	0x004010	Class length is constant
Number of elements	Number of elements in 03GINF04 file	Could be the number of genres, but unused genres are listed here
Length of one element	0x8
Class header X + 0a	Number of elements in 03GINF04 file	This information is repeated, I don't know why...

### Elements
Element content
Adress	Length	Value	Comment
X (the first adress of the element)	2 bytes	genre_id
X + 2	2 bytes	0x0100or 0x0000	0x0100 when the genre is associated to titles, 0x0000 when the genre is present in description file (03GING04) but no title of this genre exist in the player.
X + 4	2 bytes	title_id_in_TPLBlist
X + 6	2 bytes	0x0000	Constant

### Explanation
This class can't be explain without using the second class of this file and files 03GINF03 and 04CNTINF.
To make it easy, I take an example (at the right).
You can see a list of titles, the corresponding 03GINF03 and 04CNTINF files and what should be GPLB and TPLB classes.
I suppose that an album in 03GINF03 isn't used and a title in 04CNTINF isn't used.
The GPLB class gives the album_id in alphabetical order and gives the element in TPLB list of the first title of the corresponding album (see example to understand).
When there are albums listed in 03GINF03 which are not used, there are put at the end of GPLB list with constant 0x0000 instead of 0x0100.
So in GPLB, there is the list of used albums in alphabetical order and then the list of unused artist (I suppose also in alphabetical order, but this is certainly not important)
Exemple of title :
(Artist – Album – Track number – Track title – Genre)
Archive – Noise – 02 – Fuck U – Trip Hop
Archive – Noise – 03 – Waste – Trip Hop
Archive – Take My Head – 01 – You make me feel – Trip Hop
Archive – Take My Head – 02 – The way you love me – Trip Hop
Archive – Take My Head – 07 – Cloud in the sky  – Trip Hop
Radiohead – Amnesiac – 02 – Pyramid song – Rock
Radiohead – Amnesiac – 05 – I migth be wrong – Rock
Radiohead – Amnesiac – 06 – Knives out – Rock
Radiohead – Amnesiac – 09 – Hunting Bears – Rock
04CNTINF:
Element01: Radiohead – Amnesiac – 02 – Pyramid song
Element02: Radiohead – Amnesiac – 05 – I migth be wrong
Element03: Radiohead – Amnesiac – 06 – Knives out
Element04: Radiohead – Amnesiac – 09 – Hunting Bears
Element05: Archive – Take My Head – 01 – You make me feel
Element06: Archive – Take My Head – 02 – The way you love me
Element07: Archive – Take My Head – 07 – Cloud in the sky
Element08: unused
Element09: Archive – Noise – 02 – Fuck U
Element0a: Archive – Noise – 03 – Waste
03GINF04:
Element1: Trip Hop
Element2: Rock
Element3: Punk (unused)
GPLB:
Element1: 02 0x0100 01
Element2: 01 0x0100 05
Element3: 03 0x0000 00
Can be translated to :
- The first genre is the element 02 of 03GINF04 (=Trip Hop) and its first title in TPLB list is the element 01
- The second genre is the element 01 of 03GINF04 (=Rock) and its first title in TPLB list is the element 05
- The third album is the element 03 of 03GINF04 (=Punk) but it is unused.
TPLB:
Element01: 04 (= Rock – Hunting Bears [Amnesiac – 09 – Radiohead])
Element02: 02 (= Rock – I migth be wrong [Amnesiac – 05 – Radiohead])
Element03: 03 (= Rock – Knives out [Amnesiac – 06 – Radiohead])
Element04: 01 (= Rock – Pyramid song [Amnesiac – 02 – Radiohead])
Element05: 07 (= Trip Hop – Cloud in the sky [Take My Head – 07 – Archive])
Element06: 09 (= Trip Hop – Fuck U [Noise – 02 – Archive])
Element07: 06 (= Trip Hop – The way you love me [Take My Head – 02 – Archive])
Element08: 0a (= Trip Hop – Waste [Noise – 03 – Archive])
Element09: 05 (= Trip Hop – You make me feel [Take My Head – 01 – Archive])
Here, titles are listed in "Genre alphabetical order – Title name order".
The number given is title_id corresponding to the place of the title in 04CNTINF.
### Adding titles
This is explained in "Adding titles" section of TPLB class.

### Deleting titles
This is explained in "Deleting titles" section of TPLB class.

## Class 2
Class name	"TPLB"
Start adress	0x004040	Start adress is constant as previous class length is constant
Length	(Number_of_elements * Length_of_one_element) + 0x10
Number of elements	Number of used titles in the player
Length of one element	0x2
Class header X + 0a	Number of used titles in the player	This information is repeated, I don't know why...

### Elements
Element content
Adress	Length	Value	Comment
X (the first adress of the element)	2 bytes	title_id

### Explanation
As GPLB class, this class can't be explained without 03GINF04 and 04CNTINF files and GPLB.
See GPLB example as example.
This class gives all the title_id in "genre alphabetical order – title name alphabetical order" whatever the name of the artist and the name of the album (or the track number).
In this class, only used track are listed.

### Adding titles
When you add a title, you have to rebuild entirely this file (not necessarly GPLB class but always TPLB list). First, you sort titles in "the good order" = "genre alphabetical order – title name alphabetical order" and you write the title_id according to this sort in TPLB class. Then, to fill in GPLB class, you list all the used genres (in alphabetical order) and for each (in this order) you write the genre_id, the 0x0100 constant and the title_id_in_TPLBlist. Then you write all the unused genres with the 0x0000 constant.

### Deleting titles
To delete a title, you have to make the exactly same thing as "Adding titles".

*/

/*

# 01TREE22

## Table header
Table name	"TREE"
Number of classes	2 bytes

## Class 1
Class name	"GPLB"
Start adress	0x000030
Length	0x10
Number of elements	0	This file doesn't need to be filled
Length of one element	0x8

### Elements
Element content
Adress	Length	Value	Comment

### Explanation
We don't use this file for the moment

## Class 2
Class name	"TPLB"
Start adress	0x000040
Length	0x10
Number of elements	0	This file doesn't need to be filled
Length of one element	0x2

### Elements
Element content
Adress	Length	Value	Comment

### Explanation
We don't use this file for the moment

*/

/*

# 01TREE2D

## Table header
Table name	"TREE"
Number of classes	2 bytes

## Class 1
Class name	"GPLB"
Start adress	0x000030
Length	0x004010	Class length is constant
Number of elements	Number of elements in 03GINF2D file	Could be the number of albums + the number of artists + 1, but unused albums are listed here
Length of one element	0x8
Class header X + 0a	Number of elements in 03GINF2D file	This information is repeated, I don't know why...

### Elements
Element content
Adress	Length	Value	Comment
X (the first adress of the element)	2 bytes	item_id_in_03GINF2D
X + 2	2 bytes	0x0100or 0x0200or 0x0000	0x0100 when the item is associated to an artist, 0x0200 when the item is associated to an album, 0x0000 when the item is present in description file (03GING2D) but not used.
X + 4	2 bytes	title_id_in_TPLBlist
X + 6	2 bytes	0x0000	Constant

### Explanation
This class can't be explain without using the second class of this file and files 03GINF2D and 04CNTINF.
To make it easy, I take an example (at the right).
You can see a list of titles, the corresponding 03GINF2D and 04CNTINF files and what should be GPLB and TPLB classes.
I suppose that an album in 03GINF2D isn't used and a title in 04CNTINF isn't used.
The GPLB and TPLB classes list all the artists and their albums (artist alphabetical order – album alphabetical order – track number order).
The GPLB class gives the item_id_in_03GINF2D in the good order (artist1, album1_of_artist1, album2_of_artist1, artist2, album1_of_artist2,...) and gives the element in TPLB list of the first title of the corresponding album (see example to understand).
When there are items listed in 03GINF2D which are not used, there are put at the end of GPLB list with constant 0x0000 instead of 0x0100 which means the item is an artist, or 0x0200 which means the item is an album.
So in GPLB, there is the list of used items and then the list of unused items (I suppose also in the good order, but this is certainly not important)
Exemple of title :
(Artist – Album – Track number – Track title – Genre)
Archive – Noise – 02 – Fuck U – Trip Hop
Archive – Noise – 03 – Waste – Trip Hop
Archive – Take My Head – 01 – You make me feel – Trip Hop
Archive – Take My Head – 02 – The way you love me – Trip Hop
Archive – Take My Head – 07 – Cloud in the sky  – Trip Hop
Radiohead – Amnesiac – 02 – Pyramid song – Rock
Radiohead – Amnesiac – 05 – I migth be wrong – Rock
Radiohead – Amnesiac – 06 – Knives out – Rock
Radiohead – Amnesiac – 09 – Hunting Bears – Rock
04CNTINF:
Element01: Radiohead – Amnesiac – 02 – Pyramid song
Element02: Radiohead – Amnesiac – 05 – I migth be wrong
Element03: Radiohead – Amnesiac – 06 – Knives out
Element04: Radiohead – Amnesiac – 09 – Hunting Bears
Element05: Archive – Take My Head – 01 – You make me feel
Element06: Archive – Take My Head – 02 – The way you love me
Element07: Archive – Take My Head – 07 – Cloud in the sky
Element08: unused
Element09: Archive – Noise – 02 – Fuck U
Element0a: Archive – Noise – 03 – Waste
03GINF2D:
Element1: (empty)
Element2: Archive
Element3: Noise
Element4: Take My Head
Element5: Offspring (unused)
Element6: Smash (unused)
Element7: Radiohead
Element8: Amnesiac
GPLB:
Element1: 01 0x0100
Element2: 02 0x0100
Element3: 03 0x0200 01
Element4: 04 0x0200 03
Element5: 07 0x0100
Element6: 08 0x0200 06
Element7: 05 0x0000
Element8: 06 0x0000
Can be translated to :
- The first item is the element 01 of 03GINF2D, it's an artist (but I noticed that the first element of this list is always blank)
- The second item is the element 02 of 03GINF2D (=Archive), it's an artist
- The third item is the element 03 of 03GINF2D (=Noise), it's an album and its first title in TPLB list is the element 01
- The fourth item is the element 04 of 03GINF2D (=Take My Head), it's an album and its first title in TPLB list is the element 03
- The fifth item is the element 07 of 03GINF2D (=Radiohead), it's an artist
- The sixth item is the element 08 of 03GINF2D (=Amnesiac), it's an album and its first title in TPLB list is the element 06
- The seventh item is the element 05 of 03GINF2D but it is unused.
- The eighth item is the element 05 of 03GINF2D but it is unused.
TPLB:
Element01: 09 (= Archive – Noise – 02 – [Fuck U – Trip Hop])
Element02: 0a (= Archive – Noise – 03 – [Waste – Trip Hop])
Element03: 05 (= Archive – Take My Head – 01 – [You make me feel – Trip Hop])
Element04: 06 (= Archive – Take My Head – 02 – [The way you love me – Trip Hop])
Element05: 07 (= Archive – Take My Head – 07 – [Cloud in the sky  – Trip Hop])
Element06: 01 (= Radiohead – Amnesiac – 02 – [Pyramid song – Rock])
Element07: 02 (= Radiohead – Amnesiac – 05 – [I migth be wrong – Rock])
Element08: 03 (= Radiohead – Amnesiac – 06 – [Knives out – Rock])
Element09: 04 (= Radiohead – Amnesiac – 09 – [Hunting Bears – Rock])
Here, titles are listed in "Artist alphabetical order – Album alphabetical order – Track number order".
The number given is title_id corresponding to the place of the title in 04CNTINF.

Adding titles
This is explained in "Adding titles" section of TPLB class.

Deleting titles
This is explained in "Deleting titles" section of TPLB class.

Class 2
Class name	"TPLB"
Start adress	0x004040	Start adress is constant as previous class length is constant
Length	(Number_of_elements * Length_of_one_element) + 0x10
Number of elements	Number of used titles in the player
Length of one element	0x2
Class header X + 0a	Number of used titles in the player	This information is repeated, I don't know why...

Elements
Element content
Adress	Length	Value	Comment
X (the first adress of the element)	2 bytes	title_id

Explanation
As GPLB class, this class can't be explained without 03GINF2D and 04CNTINF files and GPLB.
See GPLB example as example.
This class gives all the title_id in "artist alphabetical order – album alphabetical order – track number order".
In this class, only used track are listed.

Adding titles
When you add a title, you have to rebuild entirely this file (not necessarly GPLB class but always TPLB list). First, you sort titles in "the good order" = "artist alphabetical order – album alphabetical order – track number order" and you write the title_id according to this sort in TPLB class. Then, to fill in GPLB class, you list all the used albums (in alphabetical order) and for each (in this order) you list all the used albums (in alphabetical order), you write the item_id which represents the artist with the 0x0100 constant, then you write all the item_id which represent the albums and their title_id_in_TPLBlist. Then you write all the unused items with the 0x0000 constant.

Deleting titles
To delete a title, you have to make the exactly same thing as "Adding titles".

*/

/*

# 02TREINF

## Table header
Table name	"GTIF"
Number of classes	1

## Class
Class name	"GTFB"
Start adress	0x000020
Length	0x001f00	Class length is constant
Number of elements	0x2d
Length of one element	0x90

### Elements
Element header
Adress	Length	Value	Comment
X (the first adress of the class)	12 bytes	global_key	Is written only if the element is in used (and isn't write for element 22 that exist, but isn't used)
X + 12	2 bytes	0x00 01	Means that the element contains only one subsection, Is written only if the elemen is in used
X + 14	2 bytes	0x00 80	Means that the subsection's length is 0x80, Is written only if the elemen is in used

#### Element 1
Adress	Length	Value	Comment
0x0040	4 bytes	"TIT2"
0x0042	2 bytes	0x00 02

#### Element 2
0x00d0	4 bytes	"TIT2"
0x00d2	2 bytes	0x00 02

#### Element 3
0x0160	4 bytes	"TIT2"
0x0162	2 bytes	0x00 02

#### Element 4
0x01f0	4 bytes	"TIT2"
0x01f2	2 bytes	0x00 02

#### Element 0x5 to 0x21 are not used, i.e. there are only zeros from 0x280 to 0x12b0
Element 0x22
0x01f0	4 bytes	"TIT2"
0x01f2	2 bytes	0x00 02

Element 0x23 to 0x2c are not used
Element 0x2D
0x0190	4 bytes	"TIT2"
0x0192	2 bytes	0x00 02
0x0194		"STD_TPE1"	It's written with 16bits encoding.

### Explanation
This file seems to describes the structure of the data base. It underlines what files are in used and what are their contents. This file won't be change for the moment. 

### Adding titles
The only thing to do when adding (or removing) titles is to recalculate the global_key and to change its value everywhere it appears.

### Deleting titles
Same as Adding titles

*/

/*

# 03GINF01

## Table header
Table name	"GPIF"
Number of classes	1

## Class
Class name	"GPFB"
Start adress	0x000020
Length	(Number_of_elements * Length_of_one_element) + 0x10
Number of elements	0x_number_of_album_at_music_updated_1 + 0x_number_of_album_at_music_updated_2 + ... + 0x_number_of_album_at_music_updated_N	0x_number_of_album_at_music_update is the number of albums added decreased of the number of albums deleted
Length of one element	0x310

### Elements
Element header
Adress	Length	Value	Comment
X (the first adress of the class)	8 bytes	magic_key	The magic_key is filled in with zeros in this class 
X + 8	4 bytes	album_key	The album_key of the titles added at this shot
X + 12	4 bytes	0x00060080	This means that there are 6 parts of 0x80 bytes in the element

Element content
Adress	Length	Value	Comment
X + 16	4 bytes	"TIT2"
X + 20	2 bytes	0x0002	Constant (could describes the character encoding)
from X + 22 to X + 144	122 bytes	Album name (16-bits encoding)

X + 144	4 bytes	"TPE1"
X + 148	2 bytes	0x0002	Constant (could describes the character encoding)
from X + 150 to X + 272	122 bytes	Artist name (16-bits encoding)

X + 272	4 bytes	"TCON"
X + 276	2 bytes	0x0002	Constant (could describes the character encoding)
from X + 278 to X + 400	122 bytes	Genre (16-bits encoding)

X + 400	4 bytes	"TSOP"
X + 404	2 bytes	0x0002	Constant (could describes the character encoding)
from X + 406 to X + 528	122 bytes

X + 528	4 bytes	"PICP"
X + 532	2 bytes	0x0002	Constant (could describes the character encoding)
from X + 534 to X + 656	122 bytes

X + 656	4 bytes	"PIC0"
X + 660	2 bytes	0x0002	Constant (could describes the character encoding)
from X + 662 to X + 784	122 bytes


### Explanation
This file contains the historic of albums uploaded at each music update. See example for good explaination. Element may be not used, it corresponding to albums that have been removed from the player. It's blank element, they will be overwritten at the next music update. Blank element have an album_key egals to zero.				Example with historic showed at the right : Element 1 : Take My Head (album_key = 0x26d12) Element 2 : Amnesic (album_key = 4d691) Element 3 : Noise (album_key = 45883) Element 4 : Take My Head (album_key = 59e5b)	Title historic : 1st music update : Archive – Take My Head – 01 – You make me feel (title_key = 0x26d12) Radiohead – Amnesiac – 02 – Pyramid song (title_key = 0x24a81) Radiohead – Amnesiac – 05 – I migth be wrong (title_key = 0x28c10) 2nd music update: Archive – Noise – 02 – Fuck U (title_key = 0x245f8) Archive – Noise – 03 – Waste (title_key = 0x2128b) Archive – Take My Head – 02 – The way you love me (title_key = 0x30c85) Archive – Take My Head – 07 – Cloud in the sky  (title_key = 0x291db) 


### Adding titles
Adding titles is easy, just look at the albums you are CURRENTLY adding and add this list at the end of the file (or in blank element if there are blank element), no matter if other titles of the same album are already in the player. New element are added in blank element, or at the end of the file if are no more free space (blank element).

### Deleting titles
When you delete a title from  the player, you decrease the album_key (of the corresponding album) by the title_key of the deleted title. album_key may result a zero value, it means that the element is no longer in use, it's a free space, a blank element. 

*/

/*

# 03GINF02

## Table header
Table name	"GPIF"
Number of classes	1

## Class
Class name	"GPFB"
Start adress	0x000020
Length	(Number_of_elements * Length_of_one_element) + 0x10
Number of elements	0x_number_of_artists
Length of one element	0x90

### Elements
Element header
Adress	Length	Value	Comment
X (the first adress of the class)	8 bytes	magic_key	The magic_key is filled in with zeros in this class 
X + 8	4 bytes	artist_key	The artist_key of the artist
X + 12	4 bytes	0x00010080	This means that there is 1 part of 0x80 bytes in the element

Element content
Adress	Length	Value	Comment
X + 16	4 bytes	"TIT2"
X + 20	2 bytes	0x0002	Constant (could describes the character encoding)
from X + 22 to X + 144	122 bytes	Artist name (16-bits encoding)


### Explanation
This file contains the list of artists. This list is not in alphabetical order. The order in this file is the order in which files have been added. I don't think that the order is important if the other files effectively refer to the good artist.


### Adding titles
When you add a title, you add its artist at the end of the list if it's not already written (artist appears only once here).You have to recalculate the key with new title.

### Deleting titles
When you delete a title, you decrease the artist_key by the title_key of the deleted title. You can delete the corresponding artist only if this artist doesn't have other title in the player (i.e. the key is null). When you delete an artist, the artist after take its place, i.e. you don't leave blank element in this file.
To save time, you don't have to delete the element when the artist is no longer used, the file 01TREE02 will precise that the artist is unused. And I'm not sure that you have to decrease the key...

*/

/*

# 03GINF03

## Table header
Table name	"GPIF"
Number of classes	1

## Class
Class name	"GPFB"
Start adress	0x000020
Length	(Number_of_elements * Length_of_one_element) + 0x10
Number of elements	0x_number_of_albums
Length of one element	0x90

### Elements
Element header
Adress	Length	Value	Comment
X (the first adress of the class)	8 bytes	magic_key	The magic_key is filled in with zeros in this class 
X + 8	4 bytes	album_key	The album_key of the album
X + 12	4 bytes	0x00010080	This means that there is 1 part of 0x80 bytes in the element

Element content
Adress	Length	Value	Comment
X + 16	4 bytes	"TIT2"
X + 20	2 bytes	0x0002	Constant (could describes the character encoding)
from X + 22 to X + 144	122 bytes	Album name (16-bits encoding)


### Explanation
This file contains the list of albums. This list is not in alphabetical order. The order in this file is the order in which files have been added. I don't think that the order is important if the other files effectively refer to the good album. Generally, the order in which titles are added is the alphabetical order of the corresponding artists.

### Adding titles
When you add a title, you add its album at the end of the list if it's not already written (album appears only once here). You have to recalculate the key with new title.

### Deleting titles
When you delete a title, you can delete the corresponding album only if this album doesn't have other title in the player. When you delete an album, the album after take its place, i.e. you don't leave blank element in this file. You have to recalculate the key without the title.

*/

/*

# 03GINF04

## Table header
Table name	"GPIF"
Number of classes	1

## Class
Class name	"GPFB"
Start adress	0x000020
Length	(Number_of_elements * Length_of_one_element) + 0x10
Number of elements	0x_number_of_genres
Length of one element	0x90

### Elements
Element header
Adress	Length	Value	Comment
X (the first adress of the class)	8 bytes	magic_key	The magic_key is filled in with zeros in this class 
X + 8	4 bytes	genre_key	The genre_key of the genre
X + 12	4 bytes	0x00010080	This means that there is 1 part of 0x80 bytes in the element

Element content
Adress	Length	Value	Comment
X + 16	4 bytes	"TIT2"
X + 20	2 bytes	0x0002	Constant (could describes the character encoding)
from X + 22 to X + 144	122 bytes	Genre (16-bits encoding)


### Explanation
This file contains the list of genres. This list is not in alphabetical order. The order in this file is the order in which files have been added. I don't think that the order is important if the other files effectively refer to the good genre. Generally, the order in which titles are added is the alphabetical order of the corresponding artists.


### Adding titles
When you add a title, you add its genre at the end of the list if it's not already written (genre appears only once here). You have to recalculate the key with new title.

### Deleting titles
When you delete a title, you can delete the corresponding genre only if this genre doesn't refer to another title in the player. When you delete a genre, the genre after take its place, i.e. you don't leave blank element in this file. You have to recalculate the key without the title.

*/

/*

# 03GINF22

## Table header
Table name	"GPIF"
Number of classes	1

## Class
Class name	"GPFB"
Start adress	0x000020
Length	0x10	I don't manage this file for the moment
Number of elements	0	I don't manage this file for the moment
Length of one element	0x310

### Elements
Element header
Adress	Length	Value	Comment
X (the first adress of the class)	8 bytes
X + 8	4 bytes
X + 12	4 bytes

Element content
Adress	Length	Value	Comment

### Explanation
This file has been left empty in all my test. I don't manage it for the moment. File is existing but has 0 element, it means that it contains only the table header, the class description and the class header.


### Adding titles
Nothing has to be done when adding a title.

### Deleting titles
Nothing has to be done when deleting a title.

*/

/*

# 03GINF2D

# Table header
Table name	"GPIF"
Number of classes	1

## Class
Class name	"GPFB"
Start adress	0x000020
Length	(Number_of_elements * Length_of_one_element) + 0x10
Number of elements	0x_number_of_artists + 0x_number_of_albums + 1
Length of one element	0x110

### Elements
Element header
Adress	Length	Value	Comment
X (the first adress of the class)	8 bytes	magic_key	The magic_key is filled in with zeros in this class 
X + 8	4 bytes	album_key or 0x0	The album_key of the album when the element describes an album an 0x0 otherwise.
X + 12	4 bytes	0x00020080	This means that there are 2 parts of 0x80 bytes in the element

Element content
Adress	Length	Value	Comment
X + 16	4 bytes	"TIT2"
X + 20	2 bytes	0x0002	Constant (could describes the character encoding)
from X + 22 to X + 144	122 bytes	Artist or album name (16-bits encoding)

X + 144	4 bytes	"XSOT"
X + 148	2 bytes	0x0002	Constant (could describes the character encoding)
from X + 150 to X + 272	122 bytes	Artist or album name (16-bits encoding)


### Explanation
This file is a little freak...
The first element of this class is empty, then a element describes the first artist, the following elements describe all the albums of the first artist, then another artist, its albums and so on...
See the example to understand.
As you can see, artists and albums are grouped.
The order in this file is the order in which files have been added.
I don't think that the order is important if the other files effectively refer to the good genre.
Generally, the order in which titles are added is the alphabetical order of the corresponding artists.
Example (corresponding to the title describe at the right) :
element 1 : blank element
element 2 : Archive (key = 0x0)
element 3 : Noise (album_key = 0x27c2c0)
element 4 : Take My Head (album_key = 0x32d814)
element 5 : Radiohead (key = 0x0)
element 6 : Amnesiac (album_key = 0x21d8ef)
Exemple of title sort :
Archive – Noise – 02 – Fuck U
Archive – Noise – 03 – Waste
Archive – Take My Head – 01 – You make me feel
Archive – Take My Head – 02 – The way you love me
Archive – Take My Head – 07 – Cloud in the sky
Radiohead – Amnesiac – 02 – Pyramid song
Radiohead – Amnesiac – 05 – I migth be wrong
Radiohead – Amnesiac – 06 – Knives out
Radiohead – Amnesiac – 09 – Hunting Bears

### Adding titles
When you add a title, you add its artist and album at the end of the list if they are not already written (artist and album appear only once here).
You have to recalculate the key with new title.

### Deleting titles
When you delete a title, you can delete the corresponding artist and/or album only if they don't refer to another title in the player.
When you delete an element, the element after take its place, i.e. you don't leave blank element in this file. You have to recalculate the key without the title.

*/

/*

# 04CNTINF

## Table header
Table name	"CNIF"
Number of classes	1

## Class
Class name	"CNFB"
Start adress	0x000020
Length	(Number_of_elements * Length_of_one_element) + 0x10	max_title_id is the highest title_id (title_id is the name of a file in 10F0X)
Number of elements	0x_max_title_id
Length of one element	0x290

### Elements
Element header
Adress	Length	Value	Comment
X (the first adress of the class)	2 bytes	0x0000
X + 2	2 bytes	protection	This are exactly the same 2 bytes which describes the title protection in the EA3 tag, could be 0xFFFE for MP3 encrypted, 0x0001 for OMA with DRM or FFFF for any title without encryption
X + 4	4 bytes	file properties	This are exactly the same 4 bytes which describes the title format, bitrate and sampling rate in the EA3 tag
X + 8	4 bytes	title_key	The title_key of the title
X + 12	4 bytes	0x00050080	This means that there are 5 parts of 0x80 bytes in the element

Element content
Adress	Length	Value	Comment
X + 16	4 bytes	"TIT2"
X + 20	2 bytes	0x0002	Constant (could describes the character encoding)
from X + 22 to X + 144	122 bytes	Title name (16-bits encoding)

X + 144	4 bytes	"TPE1"
X + 148	2 bytes	0x0002	Constant (could describes the character encoding)
from X + 150 to X + 272	122 bytes	Artist name (16-bits encoding)

X + 272	4 bytes	"TALB"
X + 276	2 bytes	0x0002	Constant (could describes the character encoding)
from X + 278 to X + 400	122 bytes	Album name (16-bits encoding)

X + 400	4 bytes	"TCON"
X + 404	2 bytes	0x0002	Constant (could describes the character encoding)
from X + 406 to X + 528	122 bytes	Genre (16-bits encoding)

X + 528	4 bytes	"TSOP"
X + 532	2 bytes	0x0002	Constant (could describes the character encoding)
from X + 534 to X + 656	122 bytes


### Explanation
The list of title in this file corresponds to the real titles in "10F0X" folders.
It means that the title name "1000001b.oma" in "10F00" folder correspond to the 0x1b th (=27th) element in this file

When a title is deleted in "10F0X" folders, following files are NOT renamed, every titles keep their title_id.
And the corresponding element IS NOT erased.
So you can find in this file titles which no longer are in the device.
A title_id is free, and when adding music, free title_id are filled before "creating" new title_id (increasing max_title_id).

You can't know (just with this file) the titles which are really in the device.


Adding titles
To add title, first you must know which titles are valid.
Then you sort all titles to be added, they sould be grouped by Artist (different titles of the same artist are grouped by album).
We use alphabetical order (execpt for title in album, they are sort by track number.
(See exemple at the right).
Then, you can add the elements, overwriting unused elements.
Of course, the tracks must be uploaded in the same way in "10F0X" folders, as this file corresponding to the title_ids.
Exemple of title sort :
(Artist – Album – Track number – Track title)
Archive – Noise – 02 – Fuck U
Archive – Noise – 03 – Waste
Archive – Take My Head – 01 – You make me feel
Archive – Take My Head – 02 – The way you love me
Archive – Take My Head – 07 – Cloud in the sky
Radiohead – Amnesiac – 02 – Pyramid song
Radiohead – Amnesiac – 05 – I migth be wrong
Radiohead – Amnesiac – 06 – Knives out
Radiohead – Amnesiac – 09 – Hunting Bears  Archive is before
Radiohead, Noise is before Take My Head and 01 is before 02

Deleting titles
You have nothing to do when you remove titles.
But...
I don't know what SonicStage do when removing titles with the highest title_id...
indeed, erasing these elements can save place on the device as it reduces the size of this file...

*/

/*

# 05CIDLST

## Table header
Table name	"CIDL"
Number of classes	1

## Class
Class name	"CILB"
Start adress	0x000020
Length	(Number_of_elements * Length_of_one_element) + 0x10
Number of elements	Number of titles (used or not)
Length of one element	0x30

### Elements
Element content
Adress	Length	Value	Comment
X (the first adress of the class)	0x18 bytes	drmed_oma_key

### Explanation
This file list all the title present in 04CNTINF, and gives for each title the drmed_oma_key. This value is read from title with LSI drm.

### Adding titles
This file must list all the title_ref in the order of the title in 04CNTINF.
If a title has been added at the end of O4CNTINF, its title_ref must be added at the end of this file.
If a title has been added (crushing the one before)  into 04CNTINF, its title_ref must be added at the same place in this file.

### Deleting titles
As you don't have to do anything when a title is deleted for 04CNTINF, you don't have to do anything to this file.

*/
