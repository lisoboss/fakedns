build:
	cat deploy/domain_c.conf /Volumes/WorkSpace/CLionProjects/Bronzify/deploy/share/domain_proxy_list.txt | rg -v '^regexp:' | sed 's/^full://' | sort | uniq > deploy/domain.conf
	cargo br

publish:
	sudo cp deploy/domain.conf /etc/fakedns
	sudo cp deploy/com.fakedns.plist /Library/LaunchDaemons
	sudo cp target/release/fakedns /usr/local/bin


stop:
	sudo launchctl unload /Library/LaunchDaemons/com.fakedns.plist 


r: build publish
	sudo launchctl unload /Library/LaunchDaemons/com.fakedns.plist 
	sudo launchctl load /Library/LaunchDaemons/com.fakedns.plist 
	sudo dscacheutil -flushcache
	sudo killall -HUP mDNSResponder

