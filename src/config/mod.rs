use clap::Parser;

#[derive(Debug, Parser, Clone)]
pub struct Config {
    #[clap(long, env)]
    pub api_key: String,
    #[clap(long, env, default_value = "/data/uploads")]
    pub upload_dir: String,
    #[clap(
        long,
        env,
        value_delimiter = ',',
        default_value = "Hugo_Symbol,Chromosome,Start_Position,End_Position,Variant_Classification,Variant_Type,Reference_Allele,Tumor_Seq_Allele1,Tumor_Seq_Allele2,Tumor_Sample_Barcode"
    )]
    pub required_omics_columns: Vec<String>,
}
