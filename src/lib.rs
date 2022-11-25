use std::collections::HashMap;
use std::net::{TcpStream};
use std::io::{BufReader,BufRead,Write, Read};

pub struct WildDocResult{
    body:Vec<u8>
    ,options_json:String
}
impl WildDocResult{
    pub fn body(&self)->&[u8]{
        &self.body
    }
    pub fn options_json(&self)->&str{
        &self.options_json
    }
}

pub struct WildDocClient{
    document_root:String
    ,sock:TcpStream
}
impl WildDocClient{
    pub fn new(host:&str,port:&str,document_root:&str,dbname:&str)->Self{
        let mut sock=TcpStream::connect(&(host.to_owned()+":"+port)).expect("failed to connect server");
        sock.set_nonblocking(false).expect("out of service");
        let document_root=std::path::Path::new(&(document_root.to_owned()+dbname)).to_str().unwrap().to_owned();
        sock.write_all(dbname.as_bytes()).unwrap();
        sock.write_all(&[0]).unwrap();

        let mut sig=Vec::new();
        let mut reader=BufReader::new(&sock);
        reader.read_until(0,&mut sig).unwrap();

        Self{
            document_root
            ,sock
        }
    }
    pub fn exec(&mut self,xml:&str,input_json:&str)->std::io::Result<WildDocResult>{
        let mut include_cache=HashMap::new();
        
        if input_json.len()>0{
            self.sock.write_all(input_json.as_bytes())?;
        }
        self.sock.write_all(&[0])?;

        self.sock.write_all(xml.as_bytes())?;
        self.sock.write_all(&[0])?;

        let mut reader=BufReader::new(self.sock.try_clone().unwrap());
        loop{
            let mut recv_include = Vec::new();
            if reader.read_until(0,&mut recv_include)? > 0 {
                if recv_include.starts_with(b"include:"){
                    recv_include.remove(recv_include.len()-1);
                    if let Ok(str)=std::str::from_utf8(&recv_include){
                        let s: Vec<&str>=str.split(':').collect();
                        let path=self.document_root.to_owned()+s[1];
                        let path=path.trim().to_owned();
                        let include_xml=include_cache.entry(path).or_insert_with_key(|path|{
                            match std::fs::File::open(path){
                                Ok(mut f)=>{
                                    let mut contents=Vec::new();
                                    let _=f.read_to_end(&mut contents);
                                    contents
                                }
                                ,_=>{
                                    b"".to_vec()
                                }
                            }
                        });
                        include_xml.push(0);
                        self.sock.write_all(&include_xml)?;
                    }else{
                        break;
                    }
                }else{
                    break;
                }
            }else{
                break;
            }
        }
        let mut recv_body=Vec::new();
        reader.read_until(0,&mut recv_body)?;
        recv_body.remove(recv_body.len()-1);
    
        let mut recv_options=Vec::new();
        reader.read_until(0,&mut recv_options)?;
        recv_options.remove(recv_options.len()-1);

        Ok(WildDocResult{
            body:recv_body
            ,options_json:String::from_utf8(recv_options).unwrap_or("".to_owned())
        })
    }
}
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let mut client=WildDocClient::new("localhost","51818","./test/","test");
        client.exec(r#"<wd><wd:session name="hoge">
            <wd:update commit="1">
                <collection name="person">
                    <field name="name">Noah</field>
                    <field name="country">US</field>
                </collection>
                <collection name="person">
                    <field name="name">Liam</field>
                    <field name="country">US</field>
                </collection>
                <collection name="person">
                    <field name="name">Olivia</field>
                    <field name="country">UK</field>
                </collection>
            </wd:update>
        </wd:session></wd>"#,"").unwrap();
        
        /*
        client.exec(r#"<wd>
            include-test:<wd:include src="hoge.xml" />
            <wd:search name="p" collection="person">
            </wd:search>
            OK
            <wd:result var="q" search="p">
                <div>
                    find <wd:print wd:value="wd.v('q').length" /> persons.
                </div>
                <ul>
                    <wd:for var="r" index="i" wd:in="wd.v('q')"><li>
                        <wd:print wd:value="wd.v('r').row" /> : <wd:print wd:value="wd.v('r').field('name')" /> : <wd:print wd:value="wd.v('r').field('country')" />
                    </li></wd:for>
                </ul>
            </wd:result>
        </wd>"#);

        
        client.exec(r#"<wd>
            <wd:search name="p" collection="person">
                <field name="country" method="match" value="US" />
            </wd:search>
            <wd:result var="q" search="p">
                <div>
                    find <wd:print wd:value="wd.v('q').length" /> persons from the US.
                </div>
                <ul>
                    <wd:for var="r" index="i" wd:in="wd.v('q')"><li>
                        <wd:print wd:value="wd.v('r').row" /> : <wd:print wd:value="wd.v('r').field('name')" /> : <wd:print wd:value="wd.v('r').field('country')" />
                    </li></wd:for>
                </ul>
            </wd:result>
        </wd>"#);
        client.exec(r#"<wd>
            <wd:script>
                const ymd=function(){
                    const now=new Date();
                    return now.getFullYear()+"-"+(now.getMonth()+1)+"-"+now.getDate();
                };
                const uk="UK";
            </wd:script>
            <wd:search name="p" collection="person">
                <field name="country" method="match" wd:value="uk" />
            </wd:search>
            <wd:result var="q" search="p">
                <div>
                    <wd:print wd:value="ymd()" />
                </div>
                <div>
                    find <wd:print wd:value="wd.v('q').length" /> persons from the <wd:print wd:value="uk" />.
                </div>
                <ul>
                    <wd:for var="r" index="i" wd:in="wd.v('q')"><li>
                        <wd:print wd:value="wd.v('r').row" /> : <wd:print wd:value="wd.v('r').field('name')" /> : <wd:print wd:value="wd.v('r').field('country')" />
                    </li></wd:for>
                </ul>
            </wd:result>
        </wd>"#);
        */
        client.exec(r#"<wd><wd:session name="hoge">
            <wd:update commit="1">
                <wd:search name="person" collection="person"></wd:search>
                <wd:result var="q" search="person">
                    <wd:for var="r" index="i" wd:in="wd.v('q')">
                        <collection name="person" wd:row="wd.v('r').row">
                            <field name="name">Renamed <wd:print wd:value="wd.v('r').field('name')" /></field>
                            <field name="country"><wd:print wd:value="wd.v('r').field('country')" /></field>
                        </collection>
                    </wd:for>
                </wd:result>
            </wd:update>
        </wd:session></wd>"#,"").unwrap();
        let r=client.exec(r#"<wd>
            <wd:search name="p" collection="person"></wd:search>
            <wd:result var="q" search="p">
                <div>
                    find <wd:print wd:value="wd.v('q').length" /> persons.
                </div>
                <ul>
                    <wd:for var="r" index="i" wd:in="wd.v('q')"><li>
                        <wd:print wd:value="wd.v('r').row" /> : <wd:print wd:value="wd.v('r').field('name')" /> : <wd:print wd:value="wd.v('r').field('country')" />
                    </li></wd:for>
                </ul>
            </wd:result>
        </wd>"#,"").unwrap();
        println!("{}",std::str::from_utf8(&r.body()).unwrap());
    }
}
