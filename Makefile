update-data:
	deploy/scripts/update_data.sh

build: update-data
	cargo br
	deploy/scripts/build_conf.sh

install: stop
	sudo cp -f deploy/conf.d/domain.conf /etc/fakedns/domain.conf
	sudo cp -f deploy/conf.d/domain_exclude.conf /etc/fakedns/domain_exclude.conf
	sudo cp -f deploy/conf.d/domain_block.conf /etc/fakedns/domain_block.conf
	sudo cp -f deploy/launch/com.fakedns.plist /Library/LaunchDaemons/com.fakedns.plist
	sudo cp -f target/release/fakedns /usr/local/bin/fakedns

stop:
	sudo launchctl list | grep -q com.fakedns  && sudo launchctl unload /Library/LaunchDaemons/com.fakedns.plist
	
publish: build install
	sudo launchctl load /Library/LaunchDaemons/com.fakedns.plist 
	sudo dscacheutil -flushcache
	sudo killall -HUP mDNSResponder
