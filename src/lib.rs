use std::collections::HashMap;
use std::net::{TcpStream};
use std::io::{BufReader,BufRead,Write, Read};

pub struct WildDocClient{
    document_root:String
    ,sock:TcpStream
}
impl WildDocClient{
    pub fn new(document_root:&str)->Self{
        let sock=TcpStream::connect("localhost:51818").expect("failed to connect server");
        sock.set_nonblocking(false).expect("out of service");
        Self{
            document_root:std::path::Path::new(document_root).to_str().unwrap().to_owned()
            ,sock
        }
    }
    pub fn exec(&mut self,xml:&str)->Vec<u8>{
        let mut include_cache=HashMap::new();
        match self.sock.try_clone().unwrap().write_all(xml.as_bytes()){
            Ok(())=>{
                self.sock.try_clone().unwrap().write(&[0]).unwrap();
            }
            ,Err(v) => println!("send message failed:{}",v)
        }
        loop{
            let mut recv_include = Vec::new();
            let mut reader = BufReader::new(&self.sock);
            if let Ok(v) = reader.read_until(0,&mut recv_include) {
                if v > 0 {
                    if recv_include.starts_with(b"include:"){
                        recv_include.remove(recv_include.len()-1);
                        if let Ok(str)=std::str::from_utf8(&recv_include){
                            let s: Vec<&str>=str.split(':').collect();
                            let path=self.document_root.to_owned()+s[1];
                            let path=path.trim().to_owned();
                            let xml=include_cache.entry(path).or_insert_with_key(|path|{
                                match std::fs::File::open(path){
                                    Ok(mut f)=>{
                                        let mut contents=String::new();
                                        let _=f.read_to_string(&mut contents);
                                        contents
                                    }
                                    ,_=>{
                                        "".to_string()
                                    }
                                }
                            });
                            match self.sock.try_clone().unwrap().write_all(xml.as_bytes()){
                                Ok(())=>{
                                    self.sock.try_clone().unwrap().write(&[0]).unwrap();
                                }
                                ,Err(v) => println!("send message failed:{}",v)
                            }
                        }
                    }else{
                        break;
                    }
                }
            }
        }
        let mut recv_response = Vec::new();
        let mut reader = BufReader::new(&self.sock);
        if let Ok(v) = reader.read_until(0,&mut recv_response) {
            if v > 0 {
                println!("response:{}",std::str::from_utf8(&recv_response).unwrap());
            }
        }
        recv_response
    }
}
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let mut client=WildDocClient::new("./test/");
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
        </wd:session></wd>"#);
        
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
        </wd:session></wd>"#);
        client.exec(r#"<wd>
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
        </wd>"#);
    }
}
