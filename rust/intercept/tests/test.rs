use intercept::collector::{EventCollector, EventCollectorOnTcp};
use intercept::reporter::{Reporter, TcpReporter};
use intercept::*;

mod test {
    use super::*;
    use crossbeam_channel::bounded;
    use lazy_static::lazy_static;
    use std::collections::HashMap;
    use std::io::Cursor;
    use std::path::PathBuf;
    use std::sync::Arc;
    use std::thread;
    use std::time::Duration;

    // Test that the TCP reporter and the TCP collector work together.
    // We create a TCP collector and a TCP reporter, then we send events
    // to the reporter and check if the collector receives them.
    //
    // We use a bounded channel to send the events from the reporter to the
    // collector. The collector reads the events from the channel and checks
    // if they are the same as the original events.
    #[test]
    fn tcp_reporter_and_collectors_work() {
        let collector = EventCollectorOnTcp::new().unwrap();
        let reporter = TcpReporter::new(collector.address().0).unwrap();

        // Create wrapper to share the collector across threads.
        let thread_collector = Arc::new(collector);
        let main_collector = thread_collector.clone();

        // Start the collector in a separate thread.
        let (input, output) = bounded(EVENTS.len());
        let receiver_thread = thread::spawn(move || {
            thread_collector.collect(input).unwrap();
        });
        // Send events to the reporter.
        for event in EVENTS.iter() {
            let result = reporter.report(event.clone());
            assert!(result.is_ok());
        }

        // Call the stop method to stop the collector. This will close the
        // channel and the collector will stop reading from it.
        thread::sleep(Duration::from_secs(1));
        main_collector.stop().unwrap();

        // Empty the channel and assert that we received all the events.
        let mut count = 0;
        for envelope in output.iter() {
            assert!(EVENTS.contains(&envelope.event));
            count += 1;
        }
        assert_eq!(count, EVENTS.len());
        // shutdown the receiver thread
        receiver_thread.join().unwrap();
    }

    // Test that the serialization and deserialization of the Envelope works.
    // We write the Envelope to a buffer and read it back to check if the
    // deserialized Envelope is the same as the original one.
    #[test]
    fn read_write_works() {
        let mut writer = Cursor::new(vec![0; 1024]);
        for envelope in ENVELOPES.iter() {
            let result = Envelope::write_into(envelope, &mut writer);
            assert!(result.is_ok());
        }

        let mut reader = Cursor::new(writer.get_ref());
        for envelope in ENVELOPES.iter() {
            let result = Envelope::read_from(&mut reader);
            assert!(result.is_ok());
            assert_eq!(result.unwrap(), envelope.clone());
        }
    }

    lazy_static! {
        static ref ENVELOPES: Vec<Envelope> = vec![
            Envelope {
                rid: ReporterId(1),
                timestamp: 0,
                event: Event::Started {
                    pid: ProcessId(1),
                    execution: Execution {
                        executable: PathBuf::from("/usr/bin/ls"),
                        arguments: vec!["ls".to_string(), "-l".to_string()],
                        working_dir: PathBuf::from("/tmp"),
                        environment: HashMap::new(),
                    },
                },
            },
            Envelope {
                rid: ReporterId(1),
                timestamp: 0,
                event: Event::Terminated { status: 0 },
            },
            Envelope {
                rid: ReporterId(1),
                timestamp: 0,
                event: Event::Signaled { signal: 15 },
            },
        ];
        static ref EVENTS: Vec<Event> = ENVELOPES.iter().map(|e| e.event.clone()).collect();
    }
}
