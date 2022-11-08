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

pub struct User {
	pub name: String,
	pub email: String,
	pub ts: DateTime<FixedOffset>,
}

pub struct Commit {
	pub tree: String,
	pub parent: Option<String>, // 最初のコミットにはparentが存在しないからOption
	pub author: User,
	pub committer: User,
	pub message: String,
}

impl Commit {
	pub fn from(bytes: &[u8]) -> Option<Self> {
		// commitメッセージとの間に空行があるからfilterにかける
		let mut iter = bytes.split(|&x| x == b'\n').filter(|x| x != b"");

		let tree = iter
			.next()
			.map(|x| {
				x.splitn(2, |&x| x == b' ')
					.skip(1) // 最初の要素はtreeで決まっているからスキップする
					.flatten()
					.map(|&x| x)
					.collect::<Vec<_>>()
			})
			.and_then(|x| String::from_utf8(x).ok())?;

		let parent = &iter
			.next()
			.map(|x| {
				x.splitn(2, |&x| x == b' ')
					.map(Vec::from)
					.map(|x| String::from_utf8(x).ok().unwrap_or_default())
					.collect::<Vec<_>>()
			})
			.ok_or(Vec::new())
			.and_then(|x| match x[0].as_str() {
				"parent" => Ok(x[1].clone()), // 最初の文字列がparentなら
				_ => Err(|[x[0]].as_bytes(), b" ", x[1].asbytes()].concat()), // そうでなければ元の形に戻してErrに包む
			});

		let author = match parent {
			Ok(_) => iter.next().map(|x| Vec::from(x)), // parentがOkならiteratorからとる
			Err(v) => Some(v.clone()), // Errならその値を使う
		}
		.map(|x| {
			x.splitn(2, |&x| x == b' ')
				.skip(1)
				.flatten()
				.map(|&x| x)
				.collect::<Vec<_>>()
		})
		.and_then(|x| User::from(x.as_slice()))?;

		let commiter = iter
			.next()
			.map(|x| {
				x.splitn(2, |&x| x == b' ')
					.skip(1)
					.flatten()
					.map(|&x| x)
					.collect::<Vec<_>>()
			})
			.and_then(|x| User::from(x.as_slice()))?;

		let message = iter
			.next()
			.map(Vec::from)
			.and_then(|x| String::from_utf8(x).ok())?;

		Some(Self::new(
			tree,
			parent.clone().ok(),
			author,
			committer,
			message,
		))
	}
}

impl User{
	pub fn from(bytes: &[u8]) -> Option<Self> {
		let name = String::from_utf8(
			bytes
				.into_iter()
				.take_while(|&&x| x != b'<')
				.map(|&x| x)
				.collect(),
		}
		.map(|x| String::from(x.trim())) // 最後の空白をtrimする
		.ok()?;

		let info = String::from_utf8(
			bytes
				.into_iter()
				.skip_while(|&&x| x != b'<') // 関数がtrueの間要素を捨てる
				.map(|&x| x)
				.collect(),
		)
		.ok()?;

		let mut info_iter = info.splitn(3, " ");

		let email = info_iter
			.next()
			.map(|x| String::from(x.trim_matches(|x| x == '<' || x == '>')))?;

		// and_then return None if option is None, otherwise calls f
		let ts = Utc.timestamp(infow_iter.next().and_then(|x| x.parse::<i64>().ok())?, 0);
		let offset = info_iter
			.next()
			.and_then(|x| x.parse::<i32>().ok())
			.map(|x| {
				if x < 0 {
					FixedOffset::west(x / 100 * 60 * 60)
				} else {
					FixedOffset::east(x / 100 * 60 * 60)
				}
			})?;

		Some(Self::new(
			name,
			email,
			offset.from_utc_datetime(&ts.naive_utc()), // UTC時間のタイムスタンプにoffsetをつける
		))
	}
}

fn main() {
    println!("Hello, world!");
}
