use reqwest::Result;
use serde::Deserialize;
use serde_json;

#[derive(Deserialize, Debug)]
struct ApiResponse {
    dependencies: Vec<Dependency>,
    meta: Meta,
}

#[derive(Deserialize, Debug)]
struct Dependency {
    crate_id: String,
}

#[derive(Deserialize, Debug)]
struct Meta {
    total: u32,
}

struct ReverseDependencies {
    crate_id: String,
    dependencies: <Vec<Dependency> as IntoIterator>::IntoIter,
    client: reqwest::blocking::Client,
    page: u32,
    per_page: u32,
    total: u32,
}

impl ReverseDependencies {
    fn of(id: &str) -> Result<Self> {
        Ok(Self {
            crate_id: id.to_string(),
            dependencies: vec![].into_iter(),
            client: reqwest::blocking::Client::new(),
            page: 1,
            per_page: 10,
            total: 0,
        })
    }

    fn try_next(&mut self) -> Result<Option<Dependency>> {
        if let Some(dep) = self.dependencies.next() {
            return Ok(Some(dep));
        }

        if self.page > 0 && self.page * self.per_page >= self.total {
            return Ok(None);
        }

        self.page += 1;
        let url = format!(
            "https://crates.io/api/v1/crates/{}/reverse_dependencies?page={}&per_page={}",
            self.crate_id, self.page, self.per_page
        );

        let response = self.client.get(&url).send()?;
        // convert response to ApiResponse using serde_json
        let response: ApiResponse = serde_json::from_str(&response.text().unwrap()).unwrap();
        println!("{:?}", response);

        self.dependencies = response.dependencies.into_iter();
        self.total = response.meta.total;

        Ok(self.dependencies.next())
    }
}

impl Iterator for ReverseDependencies {
    type Item = Result<Dependency>;

    fn next(&mut self) -> Option<Self::Item> {
        match self.try_next() {
            Ok(Some(dep)) => Some(Ok(dep)),
            Ok(None) => None,
            Err(e) => Some(Err(e)),
        }
    }
}

fn main() {
    for dep in ReverseDependencies::of("serde_json").unwrap() {
        println!("reverse dependency: {}", dep.unwrap().crate_id);
    }
}
