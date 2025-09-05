update-data:
	deploy/scripts/update_data.sh

build: update-data
	deploy/scripts/build_conf.sh
	cargo br

install:
	sudo cp -f deploy/conf.d/domain.conf /etc/fakedns
	sudo cp -f deploy/conf.d/domain_exclude.conf /etc/fakedns
	sudo cp -f deploy/conf.d/domain_block.conf /etc/fakedns
	sudo cp -f deploy/launch/com.fakedns.plist /Library/LaunchDaemons
	sudo cp -f target/release/fakedns /usr/local/bin

stop:
	sudo launchctl unload /Library/LaunchDaemons/com.fakedns.plist 

publish: build install
	sudo launchctl unload /Library/LaunchDaemons/com.fakedns.plist 
	sudo launchctl load /Library/LaunchDaemons/com.fakedns.plist 
	sudo dscacheutil -flushcache
	sudo killall -HUP mDNSResponder
