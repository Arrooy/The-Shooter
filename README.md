# The Shooter
The shooter is a project made to learn about FAT and Ext2 filesystems.

It is written in [Rust](https://www.rust-lang.org/). :sparkling_heart:

This was created for the Advanced Operating Systems (ASO) final project.
## Summary - How to run the program
This project supports basic operations
with both filesystems. The available operations are: 
* **Info**: Retrieves all the available filesystem information
* **Find**: Searches (recursively) for a file inside a filesystem. Prints out the size of the file.
* **Delete**: Deletes a file from the root directory of a filesystem.

It's command-line based software. To execute the previous features, use the following commands in the project root folder:

Print to terminal filesystem information
```
cargo run /info FAT16
cargo run /info Ext2
```
Look for hello.txt filesize
```
cargo run /find FAT16 hello.txt
cargo run /find Ext2 hello.txt
```
Erase hello.txt from the filesystem
```
cargo run /delete FAT16 hello.txt
cargo run /delete Ext2 hello.txt
```
In the above commands, the first argument specifies the filesystem. The second arg (if defined) specifies the filename.
***
## Table of contents:
* [How to Compile the project](#how-to-compile-the-project)
    + [How to install rust](#how-to-install-rust)
    + [Requirements to execute](#requirements-to-execute)
    + [Linux Kernel](#linux-kernel)
* [Filesystem explanation](#filesystem-explanation)
    + [FAT](#fat)
    + [EXT2](#ext2)
* [Project description](#project-description)
    + [Requirements](#requirements)
    + [Design](#design)
    + [Data Structures](#data-structures)
    + [Tests Performed](#tests-performed)
    + [Troubleshooting](#troubleshooting)
    + [Temporal estimation](#temporal-estimation)
* [Git details](#git-details)
* [Conclusions](#conclusions)
***
## How to Compile the project 
The software uses [Rust](https://www.rust-lang.org/) language. Please follow these instructions to install the required toolchain.
### How to install RUST
Run `curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh` to install RUST.

More info [here](https://www.rust-lang.org/tools/install)
### Requirements to execute
- Rust toolchain installed:
    - (rustc 1.51.0 (2fd73fabe 2021-03-23))
    - (cargo 1.51.0 (43b129a20 2021-03-16))
### Linux Kernel
Kernel used is `Linux 5.8.0-50-generic #56~20.04.1-Ubuntu x86_64 GNU/Linux`
***
## Filesystem explanation
As mentioned before, this project manages Ext2 and FAT. In this section, let's revise how these filesystems are 
designed.

### FAT
FAT stands for File Allocation Table. It was presented in late 1970 by Microsoft.
The nature of a filesystem is to provide format (structure) to the hard disk data. By hard disk, I mean any sort of long term memory such as HDD, SSD, USB or even floppy disks.

This filesystem has 3 subtypes but this project only supports the 16-byte version. FAT16 has the following inner structure:

<img src="https://github.com/Arrooy/The-Shooter/blob/master/img/FAT16_struct.jpg" style="background: white; display: block; margin:auto;" alt="FAT Filesystem Structure">

As seen in the image, the filesystem is logically divided into 4 regions. 
These are the reserved region (first region), the FAT region (red and green in the diagram), the root directory region,
and the data region.

A FAT16 volume is divided into several clusters. These clusters contain multiple sectors inside. Every sector consists of an exact number of bytes.
The first region contains the boot sector and the BIOS parameter block. This space of memory has the specifications required to navigate
the filesystem. It fully describes de filesystem providing information such as the number of clusters, the number of sectors in a cluster, and the number of bytes per sector.

The FAT region contains multiple tables (FATS). All FATS are the same, they are backups useful in case of data loss. These tables are used when a file uses more than 1 cluster. When this happens, the file is spread through the disk using multiple clusters. Given a cluster number, the FAT provides the next cluster number where the data continues.

The Root directory region is a memory space designated to the base directory. There, the initial folder is located with all its files. A folder in FAT is a file with a list of 32byte directory entries.

Each directory entry contains information such as the filename, the filetype, some timestamps for the creation/lastWrite/lastAccess events, the filesize and the file cluster. This later one indicates the file contents location. If the file size is bigger than a cluster, after reading the cluster data, a lookup to the FAT is mandatory to discover the next cluster of the same file. The cluster number appearing in the FAT table also indicates that the file has no more data if it is bigger than 0xfff7.

Finally, the last region contains every file cluster provided by each root directory entry.

### EXT2
The Extended Filesystem 2 (EXT2) is a filesystem released in 1993 as part of the Linux Kernel. The filesystem has the following structure:

<img src="https://www.oreilly.com/library/view/understanding-the-linux/0596005652/httpatomoreillycomsourceoreillyimages9320.png" style="background: white; display: block; margin:auto;" alt="EXT2 Structure">

This time, the formatted volume has 2 main regions. The boot block and the group blocks. Inside each block group, there are several blocks of formatted data. A block is a small group of sectors on the disk. Differing from the FAT filesystem, the EXT2 parameters are located within the superblock inside the block group.

The disk is divided into different block groups to increase fragmentation and file seek speed while reading large amounts of consecutive data.

Inside each block group, there is the superblock (only applies to the block groups 0,1 and powers of 3,5,7). The superblock, as mentioned before, contains information about the configuration of the filesystem. 

Following the superblock, there are the group descriptors. These help drivers locate the inode bitmap, the inode table and the data block bitmap. It provides additional configuration data about the block group.

Next, the data block bitmap and the inode bitmap. These are bitmaps that indicate whether a block/bitmap is used or not. Each bit in the bitmap represents a block number or an inode id. If the bit is 1, the block/inode is used.

Every object in the filesystem has an index node named inode. This structure contains pointers to the filesystem blocks containing the file data. Additionally, it contains all the file metadata except the file name. 

To point to the data, there are pointers to the first 12 blocks. There is also a pointer to an indirect block. This indirect block contains a list of pointers to the file data blocks. Additionally, the inode has a double and triple indirect block. These refer accordingly to a list of indirect and double indirect blocks. 

The following diagram illustrates this indirect linkages: 

<img src="https://upload.wikimedia.org/wikipedia/commons/thumb/0/09/Ext2-inode.svg/1200px-Ext2-inode.svg.png" style="background: white; display: block; margin:auto;" alt="Inode Structure">

By this means, the filesystem has support for way bigger files compared to FAT. 

All the block group inodes are contained inside the inode table. The data blocks can be later accessed using the inode information.

***
## Project description
### Requirements
This project presents an implementation of a command-line utility to gather information from a filesystem formatted in Fat16 or Ext2. Additionally, it can delete files given a name.

As a result, the user can execute the following operations:
* **Info**: Retrieves all the available filesystem information.
* **Find**: Searches (recursively) for a file inside a filesystem. Prints out the size of the file.
* **Delete**: Deletes a file from the root directory of a filesystem.

### Design
Rust has no direct class definition. Instead, the data and the object functionalities are independent. A struct defines the data of an object. To add logic to an object, an implementation to that struct must be provided. The overall program design profits from a generic perspective of the filesystem. Since all filesystems in this project have to implement the same functionalities, a RUST trait is provided to forge a contract that all filesystem implementations must follow.  

This is the basic program flow:
Initially, the filesystem is loaded into a class named Generic Volume. This reads all the volume data and saves it into memory.
This generic volume contains an implementation providing tools to detect what type of Filesystem we are working with.

Once the filesystem type is detected, the Generic Volume is converted into FAT16 or EXT2 structures. These extract
all the information from the Generic Volume and expand it using the filesystem definition data.

Both FAT16 and EXT2 structs extend a Filesystem trait. This defines the base behaviour of the filesystem, making sure that every filesystem has the following operations: Info, delete, find. Additionally, by implementing this trait, the command-line behaviour is respected, executing each previously mentioned function when receiving the /info /find /delete commands automatically.

Delete and find commands use a recursive approach to dive into deep folder levels. This isn't optimal since it uses more resources than the iterative solution. Nevertheless, I decided to leave it recursive since it is much more readable. 

Finally, there is a module named utils that provide low-level byte functionalities to manipulate the volume raw bytes.

All of this logic lead to a very readable main demonstrates rapidly what are the software capabilities.

### Data Structures
The data structures used are 4 structs. Three of them are used by each filesystem: GenericVolume, Ext2 and FAT. These structs contain all the information related to the filesystem itself and its characteristics. 

The last struct references the result of an Ext2 search. This is used to return multiple values from a find. This way, when looking for a file I'm also able to gather the inode number and its parent directory.

### Tests Performed
To check the correct performance of each stage of the software, it has been tested exhaustively using the following techniques:

- The information gathered with /info has been contrasted with the information provided by these two commands
    - Fat `fatcat -i Fat16.fs`
    - Ext2 `dumpe2fs -h Ext2`

- Find and delete commands were tested by manually mounting the filesystem. Then, many files were added to different directory levels.
    - /find was able to locate all files no matter the size or directory depth.
    - /delete was able to delete files in the root directory. By unmounting and mounting again the fs, the data was deleted successfully.   

To accelerate this last task, a bash script (create_fat.bash) is provided to automate the fat fs testing.

Additionally, to these checks, the development has been supported by the utility **hexdump**. With this command, it's easy to see the raw bytes of the filesystem and contrast the theoretical concepts with real formatted volumes.

With `hexdump -C file.fs -s<> -n<>` the user can select the desired data to display and see the translated Ascii on the right side of the terminal.

### Troubleshooting
Multiple problems occurred developing this project, here are some of the most important ones:

During the development of the FAT info command, I found myself working with a FAT12 partition. I wasted multiple hours looking for errors in the code and in the end, my mistake was creating the filesystem without the flag `-F 16`. This flag specifies that de newly formatted file must be subversion 16. When this isn't set, the system automatically uses the best formatting, in that case, Fat12.

Additionally, when working with the delete command in the FAT16 filesystem, I realised that my search algorithms were wrong. Instead of using the theoretical methodologies that suggest working with clusters and the FAT table, I accidentally was iterating through all the volume looking for formatted data. That worked nicely but merely because I was adding and removing very few files in the fs. This way, the data was stored sequentially, and it was easy to access by looking directly at it.

Another problem that I found is when deleting files from EXT2 they don't seem deleted from my OS perspective until I remount the volume. I suppose that Ubuntu is doing some caching or optimizations that ignore mine erase. When I remount the filesystem, all the data gets refreshed and this time, the files appear correctly as deleted.

Finally, FAT has some files are stored with LongNames. The teacher explicitly said that we can ignore these entries in the volume. The problem here is that, when deleting a file that contains a long name, its short name entry gets deleted along with its data, but, since the long name data persists, ubuntu keeps thinking that the file still exists even after remounting the partition. If the user tries to access the file data, the OS shows an error indicating that there is no data there.

Apart from Filesystem problems, I also encountered typical errors when learning a new language. I fought with Rust several times since the lang has some exotic (nevertheless interesting) programming techniques. It introduces lots of programming tools and concepts that I have personally never used in Python, C, C++ or Java. Features like variable ownership and mutability were hard to adapt to, coming from other programming environments.

What does the community see RUST learning curve?

<img src="https://i.pinimg.com/originals/79/74/bb/7974bb8d1ab953614bea074d01926ef2.jpg" style="background: white; display: block; margin:auto;" alt="RUST Learning curve">

### Time investment
The project used 15 days of work. This is ~83 hours approx.
* Fase 1: 
    * Project setup + RUST learning (I read this [book](https://dhghomon.github.io/easy_rust/)): 20h
    * FAT16: 3h
    * EXT2: 1h
* Fase 2 & 3:
    * FAT16: 10h
    * EXT2: 20h
* Fase 4:
    * FAT16: 4h
    * EXT2: 6h
* Solving Bugs: 10h
* Readme: 9h

*** 
## Git details
Git is a version control system. This project is used to track changes and develop each project feature separately.
By having each project component in its isolated branch, is easier to maintain the code, since all project changes are logged and can be accessed later.
From my perspective, it was nice to have version control in the project.

On multiple occasions I found myself stuck with a software problem that I was unable to solve.
Instead of frustrating and getting tired of the work, I was able to checkout to the base branch and continue developing another feature. This way,
the project work could maintain a steady rate of progress throughout its development.
***
## Conclusions
This work provides a brief look at the foundations of modern filesystems. With the "learn by doing" methodology, I obtained extensive knowledge on FAT16 and Ext2.
As expected, the software didn't work on the first try. That forced me to debug and search way more content online than I have previously expected, increasing this way, the area of theoretical content learned.

The shooter has taught me that learning about outdated filesystems isn't a bad idea. By taking a look at how the very first filesystems were made,
I took a glimpse at the engineer's way of thinking at that time. On top of that, I discovered their design mistakes and learned why new filesystems have emerged.
Therefore, I'm now able to understand how they overcome these problems.