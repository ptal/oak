// Copyright 2018 Chao Lin & William Sergeant (Sorbonne University)

// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at

//     http://www.apache.org/licenses/LICENSE-2.0

// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

extern crate oak_runtime;
use oak_runtime::*;
use oak::oak;

oak! {
    program =  spacing json_expr spacing
    // well_formed_json = lbrace json_expr? rbrace

    //JSON
    json_expr
    = number > make_json_number
    /   json_string > make_json_string
    /   json_array > make_json_array
    /   json_object > make_json_object

    json_object
    = lbrace json_members? spacing rbrace

    json_members
    = json_pair (coma json_pair)* > make_json_member

    json_pair
    = json_string colon json_expr > make_json_pair

    json_array
    = lbracket (json_expr coma)* json_expr spacing rbracket

    json_string
    = dquote json_char* dquote spacing > to_string

    //Generic Types
    dquote = "\""
    spacing = [" \n\r\t"]*:(^)
    digit = ["0-9"]
    colon = ":" spacing
    semicolon = ";" spacing
    coma = "," spacing
    number = digit+ spacing > to_number
    not_zero_digit = ["1-9"]
    digits = digit+

    json_char
    = ["a-zA-Z0-9_() ,.:&'/@?*=;-"]

    lparen = "(" spacing
    rparen = ")" spacing
    lbracket = "[" spacing
    rbracket = "]" spacing
    lbrace = "{" spacing
    rbrace = "}" spacing

    use std::str::FromStr;

    pub type PExpr = Box<JSONExpr>;

    //Enums
    #[derive(Debug)]
    pub enum JSONExpr {
        Str(String),
        Number(u32),
        Array(Vec<Box<JSONExpr>>),
        Object(Option<Box<JSONPair>>)
    }

    #[derive(Debug)]
    pub enum JSONPair {
        Pair(String, Box<JSONExpr>),
        Json(Vec<Box<JSONPair>>)
    }

    //Functions

    fn make_json_number(number:u32)-> Box<JSONExpr> {
        Box::new(JSONExpr::Number(number))
    }

    fn make_json_string(string:String) -> Box<JSONExpr> {
        Box::new(JSONExpr::Str(string))
    }

    fn make_json_pair(string:String, expr:Box<JSONExpr>) -> Box<JSONPair> {
        Box::new(JSONPair::Pair(string,expr))
    }

    fn make_json_array(array:Vec<Box<JSONExpr>>, front:Box<JSONExpr>) -> Box<JSONExpr> {
        let mut vector = Vec::new();
        for i in array{
            vector.push(i);
        }
        vector.push(front);
        Box::new(JSONExpr::Array(vector))
    }

    fn make_json_member(pair: Box<JSONPair>, rest: Vec<Box<JSONPair>>) -> Box<JSONPair> {
        let mut vector = vec![pair];
        for i in rest{
            vector.push(i);
        }
        Box::new(JSONPair::Json(vector))
    }

    fn make_json_object(m: Option<Box<JSONPair>>) -> Box<JSONExpr> {
        Box::new(JSONExpr::Object(m))
    }

    fn to_number(raw_text: Vec<char>) -> u32 {
        u32::from_str(&*to_string(raw_text)).unwrap()
    }


    fn to_string(raw_text: Vec<char>) -> String {
        raw_text.into_iter().collect()
    }

}


// fn analyse_state(state: ParseState<StrStream, json::PExpr>)  {
//     use oak_runtime::parse_state::ParseResult::*;
//     match state.into_result() {
//         Success(data) => println!("Full match: {:?}", data),
//         Partial(data, expectation) => {
//             println!("Partial match: {:?} because {:?}", data, expectation);
//         }
//         Failure(expectation) => {
//             println!("Failure: {:?}", expectation);
//         }
//     }
// }
//
// fn main() {
//      analyse_state(json::parse_program(r##"{"ue" : "pstl" }"##.into_state())); // Complete
//      analyse_state(json::parse_program(r##"{"ue" : "pstl"} , "##.into_state())); // Partial
//      analyse_state(json::parse_program(r##"{"pstl""##.into_state())); // Error
//
//     let json =
//     r##"{
//         "ue" : "pst",
//         "note" : [20, 21, 22],
//         "enseignement" : "ptal sensei"
//     }"##;
//     let mut sjson = json.into_state();
//     analyse_state(json::parse_program(sjson));
//
//     let json_full =
//     r##"{"web-app": {
//   "servlet": [
//     {
//       "servlet-name": "cofaxCDS",
//       "servlet-class": "org.cofax.cds.CDSServlet",
//       "init-param": {
//         "configGlossary:installationAt": "Philadelphia, PA",
//         "configGlossary:adminEmail": "ksm@pobox.com",
//         "configGlossary:poweredBy": "Cofax",
//         "configGlossary:poweredByIcon": "/images/cofax.gif",
//         "configGlossary:staticPath": "/content/static",
//         "templateProcessorClass": "org.cofax.WysiwygTemplate",
//         "templateLoaderClass": "org.cofax.FilesTemplateLoader",
//         "templatePath": "templates",
//         "templateOverridePath": "",
//         "defaultListTemplate": "listTemplate.htm",
//         "defaultFileTemplate": "articleTemplate.htm",
//         "useJSP": "false",
//         "jspListTemplate": "listTemplate.jsp",
//         "jspFileTemplate": "articleTemplate.jsp",
//         "cachePackageTagsTrack": 200,
//         "cachePackageTagsStore": 200,
//         "cachePackageTagsRefresh": 60,
//         "cacheTemplatesTrack": 100,
//         "cacheTemplatesStore": 50,
//         "cacheTemplatesRefresh": 15,
//         "cachePagesTrack": 200,
//         "cachePagesStore": 100,
//         "cachePagesRefresh": 10,
//         "cachePagesDirtyRead": 10,
//         "searchEngineListTemplate": "forSearchEnginesList.htm",
//         "searchEngineFileTemplate": "forSearchEngines.htm",
//         "searchEngineRobotsDb": "WEB-INF/robots.db",
//         "useDataStore": "true",
//         "dataStoreClass": "org.cofax.SqlDataStore",
//         "redirectionClass": "org.cofax.SqlRedirection",
//         "dataStoreName": "cofax",
//         "dataStoreDriver": "com.microsoft.jdbc.sqlserver.SQLServerDriver",
//         "dataStoreUrl": "jdbc:microsoft:sqlserver://LOCALHOST:1433;DatabaseName=goon",
//         "dataStoreUser": "sa",
//         "dataStorePassword": "dataStoreTestQuery",
//         "dataStoreTestQuery": "SET NOCOUNT ON;select test='test';",
//         "dataStoreLogFile": "/usr/local/tomcat/logs/datastore.log",
//         "dataStoreInitConns": 10,
//         "dataStoreMaxConns": 100,
//         "dataStoreConnUsageLimit": 100,
//         "dataStoreLogLevel": "debug",
//         "maxUrlLength": 500}},
//     {
//       "servlet-name": "cofaxEmail",
//       "servlet-class": "org.cofax.cds.EmailServlet",
//       "init-param": {
//       "mailHost": "mail1",
//       "mailHostOverride": "mail2"}},
//     {
//       "servlet-name": "cofaxAdmin",
//       "servlet-class": "org.cofax.cds.AdminServlet"},
//
//     {
//       "servlet-name": "fileServlet",
//       "servlet-class": "org.cofax.cds.FileServlet"},
//     {
//       "servlet-name": "cofaxTools",
//       "servlet-class": "org.cofax.cms.CofaxToolsServlet",
//       "init-param": {
//         "templatePath": "toolstemplates/",
//         "log": 1,
//         "logLocation": "/usr/local/tomcat/logs/CofaxTools.log",
//         "logMaxSize": "",
//         "dataLog": 1,
//         "dataLogLocation": "/usr/local/tomcat/logs/dataLog.log",
//         "dataLogMaxSize": "",
//         "removePageCache": "/content/admin/remove?cache=pages&id=",
//         "removeTemplateCache": "/content/admin/remove?cache=templates&id=",
//         "fileTransferFolder": "/usr/local/tomcat/webapps/content/fileTransferFolder",
//         "lookInContext": 1,
//         "adminGroupID": 4,
//         "betaServer": "true"}}],
//   "servlet-mapping": {
//     "cofaxCDS": "/",
//     "cofaxEmail": "/cofaxutil/aemail/*",
//     "cofaxAdmin": "/admin/*",
//     "fileServlet": "/static/*",
//     "cofaxTools": "/tools/*"},
//
//   "taglib": {
//     "taglib-uri": "cofax.tld",
//     "taglib-location": "/WEB-INF/tlds/cofax.tld"}}}"## ;
//
//     sjson = json_full.into_state();
//     analyse_state(json::parse_program(sjson));

}
