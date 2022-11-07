pub enum GitObject {
	Blob(Blob),
	Tree(Tree),
	Commit(Commit),
}

pub struct Blob {
	pub size: usize,
	pub content: String,
}


impl Blob {
	pub fn new(content: String) -> Self {
		Self {
			size: content.len(),
			content,
		}
	}
	
	pub fn from(bytes: &[u8]) -> Option<Self> {
		let content = String::from_utf8(bytes.to_vec());
	
		match content {
			Ok(content) => Some(Self {
				size: content.len(),
				content,
			}),
			_ => None,
		}
	}
	
	pub fn as_bytes(&self) -> Vec<u8> {
		// headerとbodyが\0で区切られる
		let header = format!("blob {}\0", self.size);
		let store = format!("{}{}", header, self.to_string());
	
		Vec::from(store.as_bytes())
	}
	
	pub fn calc_hash(&self) -> Vec<u8> {
		Vec::from(Sha1::digest(&self.as_bytes()).as_slice())
	}
}

pub struct Tree {
	pub contents: Vec<File>,
}

pub struct File {
	pub mode: usize,
	pub name: String,
	pub hash: Vec<u8>,
}

impl File {
	pub fn from(header: &[u8], hash: &[u8]) -> Option<Self> {
		let split_header = String::from_utf8(header.to_vec()).ok?;

		let mut iter = split_header.split_whitespace();

		let mode = iter.next().and_then(|x| x.parse::<usize>().ok())?;
		let name = iter.next()?;

		Some(Self::new(mode, String::from(name), hash))
	}

	pub fn encode(&self) -> Vec<u8> {
		let header = format!("{} {}\0", self.mode, self.name);
		[header.as_bytes(), &self.hash].concat()
	}
}

impl Tree {
	pub fn from(bytes: &[u8]) -> Option<Self> {
		let contents: Vec<File> = Vec::new();
		let mut iter = bytes.split(|&b| b == b'\0'); // entry is splited by '\0'

		let mut header = iter.next()?;
		let contents = iter.try_fold(contents, |mut acc, x| {
			let (hash, next_header) = x.split_at(20); // hash value is 20bytes so split 20
			let file = File::from(header, hash)?;

			acc.push(file);
			header = next_header;
			Some(acc)
		})?;
		Some(Self { contents })
	}

	pub fn as_bytes(&self) -> Vec<u8> {
		let content: Vec<u8> = self.contents.iter().flat_map(|x| x.encode()).collect(); // flat_mapにわたる値がiterator(この場合にmapは使えない)
		let header = format!("tree {}\0", content.len());

		[header.as_bytes(), content.as_slice()].concat()
	}
}



fn main() {
    println!("Hello, world!");
}
