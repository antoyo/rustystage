/*
This is the data found in each header of an OMA file
Header size seems to be always 0xc60 (3160) bytes
Tags are case sensitive, numbers are little endian – high then low bytes

Adress																			Name	Comment
0	"E"	"A"	"3"	3	0	0	0	0	17	76	"T"	"I"	"T"	"2"	0	0			« TIT2 »	This is the tag for the title of the mp3/wma...
10	var1	var1	0	0	2	Title string												var1 (2 bytes)	Title length	Number of chars in the title, value is (number of chars)*2 + 1
XX	"T"	"P"	"E"	"1"	0	0	var2	var2	0	0	2	Artist string							Title string	This string is composed of two bytes of each char.  Ex: « La Bamba » would be  [0x0,L,0x0,a,0x0,SPACE,0x0,B,0x0,a,0x0,m,0x0,b,0x0,a]
XX	"T"	"A"	"L"	"B"	0	0	var3	var3	0	0	2	Album string							« TPE1 »	This is the artist tag
XX	"T"	"C"	"O"	"N"	0	0	var4	var4	0	0	2	Genre string						var2 (2 bytes)	Artist length	Number of chars in the string to read (Same thing as the « Title »)
XX	"T"	"X"	"X"	"X"	0	0	0	17	0	0	2	0	"O"	0	"M"	0			Artist string	Same as Title string
XX	"G"	0	"_"	0	"T"	0	"R"	0	"A"	0	"C"	0	"K"	0	0	var5			« TALB »	Tag of the album name
XX	var5	"T"	"Y"	"E"	"R"	0	0	0	9	0	0	2	var6	var6	var6	var6		var3 (2 bytes)	Album length	Number of chars in the string to read (Same thing as the « Title »)
XX	var6	var6	var6	var6	"T"	"X"	"X"	"X"	0	0	0	3b	0	0	2	0			Album string	Same as Title string
XX	"O"	0	"M"	0	"G"	0	"_"	0	"T"	0	"R"	0	"L"	0	"D"	0			« TCON »	Tag of the genre
XX	"A"	0	0	var6	var6	var6	var6	var6	var6	var6	var6	0	"/"	0	"0"	0		var4 (2 bytes)	Genre length	Number of chars in the string to read (Same thing as the « Title »)
XX	"1"	0	"/"	0	"0"	0	"1"	0	" "	0	"0"	0	"0"	0	":"	0			Genre string	Same as Title string
XX	"0"	0	"0"	0	":"	0	"0"	0	"0"	"T"	"L"	"E"	"N"	0	0	0			« TXXX OMG TRACK »	Tag of the track number
XX	0d	0	0	2	var7	var7	var7	var7	var7	var7	0	"0"	0	"0"	0	"0"		var5 (2 bytes)	Track number	Value is the char(s) of number of the track (i.e. if the track number is 2, these bytes must contain "00" "32")
	Filling zeros to reach the end of the header...																		« TYER »	Tag of the year
0c00	"E"	"A"	"3"	2	0	60	ff	ff	0	0	0	0	1	0f	50	0		var6 (8 bytes)	Year string	Same as Title string (format must be "AAAA")
0c10	?	?	?	?	?	?	?	?	?	?	?	?	?	?	?	?			« TLEN »	Tag of the time length of the title.  This is the number of seconds composed of 6 chars.
0c20	var8	var8	var8	var8	?	?	?	?	?	?	?	?	?	?	?	?		var7 (6 bytes)	Track lenght	The track lenght string in seconds (Same thing as the « Title »)
0c30	0	0	0	0	0	0	0	0	0	0	0	0	0	0	0	0		0c01 (16 bytes)		Normally, it is related of the encoding keys on other NW-xxxx players.  But for the NW-E00x, it is not used
0c40	0	0	0	0	0	0	0	0	0	0	0	0	0	0	0	0		var8 (4 bytes)	3,63,221,16 (MP3) 5,64,179,69 (WMA)	File format.  Thoses values are mp3 or WMA and must be validate  Would have to validate of AAC and WAV
0c50	0	0	0	0	0	0	0	0	0	0	0	0	0	0	0	0
0c60	Here starts the audio file... For MP3 the ID3 tags header should be removed...
*/
