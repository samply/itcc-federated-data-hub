pub fn write_cbio_meta_study(meta_path: &std::path::Path) -> anyhow::Result<()> {
    // Keep it simple; customize fields as you need.
    let content = r#"cancer_study_identifier: itcc
        type_of_cancer: example_cancer_type
        name: INFORM Oncoanalyzer data 
        description: INFORM OA DATASET
        groups: CSR
        reference_genome: hg38
        add_global_case_list: true
        "#;
    std::fs::write(meta_path, content)?;
    Ok(())
}

pub fn write_cbio_meta_cancer_type(meta_path: &std::path::Path) -> anyhow::Result<()> {
    let content = r#"
        genetic_alteration_type: CANCER_TYPE
        datatype: CANCER_TYPE
        data_filename: cancer_type.txt
    "#;
    std::fs::write(meta_path, content)?;
    Ok(())
}

pub fn write_cbio_meta_seg(meta_path: &std::path::Path) -> anyhow::Result<()> {
    let content = r#"
        cancer_study_identifier: itcc
        genetic_alteration_type: COPY_NUMBER_ALTERATION
        datatype: SEG
        data_filename: data_seg.seg
        description: CNA seg
        reference_genome_id: hg38
    "#;
    std::fs::write(meta_path, content)?;
    Ok(())
}

pub fn write_cbio_meta_mutation(
    meta_path: &std::path::Path,
    maf_file_name: &str,
) -> anyhow::Result<()> {
    let content = format!(
        r#"
        cancer_study_identifier: itcc
        genetic_alteration_type: MUTATION_EXTENDED
        datatype: MAF
        data_filename: {maf_file_name}
        stable_id: mutations
        profile_name: Mutations
        profile_description: WGS mutations
        show_profile_in_analysis_tab: true
        swissprot_identifier: name
    "#
    );
    std::fs::write(meta_path, content)?;
    Ok(())
}

pub fn write_cbio_meta_clinical_sample(meta_path: &std::path::Path) -> anyhow::Result<()> {
    let content = r#"
        cancer_study_identifier: 
        genetic_alteration_type: CLINICAL
        datatype: SAMPLE_ATTRIBUTES
        data_filename: data_clinical_sample.txt
    "#;
    std::fs::write(meta_path, content)?;
    Ok(())
}

pub fn write_cbio_cancer_type(meta_path: &std::path::Path) -> anyhow::Result<()> {
    let content = r#"
        example_cancer_type	Custom Cancer Type	HotPink	tissue
    "#;
    std::fs::write(meta_path, content)?;
    Ok(())
}

pub fn write_cbio_data_seg(data_path: &std::path::Path) -> anyhow::Result<()> {
    let content = r#"
        
    "#;
    std::fs::write(data_path, content)?;
    todo!();
    Ok(())
}

pub fn write_cbio_data_clinical_sample(data_path: &std::path::Path) -> anyhow::Result<()> {
    let content = r#"
        
    "#;
    std::fs::write(data_path, content)?;
    todo!();
    Ok(())
}

pub fn write_cbio_data_clinical_patient(data_path: &std::path::Path) -> anyhow::Result<()> {
    let content = r#"
        
    "#;
    std::fs::write(data_path, content)?;
    todo!();
    Ok(())
}
