scp -r deploy strawlab-org:strawlab.org/ethotimer.new
ssh strawlab-org "rm -rf strawlab.org/ethotimer && mv strawlab.org/ethotimer.new strawlab.org/ethotimer"
