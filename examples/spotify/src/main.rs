fn main() {
    let mut pipeline = PipelineBuilder::new()
        .pipe(StdoutOutput::new())
        .build();

    let mut connection = Connection::new(pipeline);

    connection.set_source(ByteStreamSource::from([some byte array idk]));
    connection.add_output::<StdoutOutput>();

    // This will keep running until the source is exhausted, runs on sample rate cycles through the pipeline.
    connection.run();

    // Will pause any future reading/streaming from the source.
    connection.pause();

    // Will resume reading/streaming from the source.
    connection.resume();

    // Will make output updates happen on the next sample rate cycle.
    connection.remove_output::<StdoutOutput>();
    connection.add_output::<StdoutOutput>();
}
