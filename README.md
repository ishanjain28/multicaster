# Multicaster

This is a software for routing multicast traffic across L2 networks.
It'll allow you to be very specific about the exact traffic that is sent over.


# Working Notes

1. It needs to listen on the specified port to receive multicast traffic.
This causes problems if there are other softwares that are also listening without using `SO_ADDR_REUSE`.

For now, Disable those softwares when running this. A list of such softwares,

a. avahi-daemon


### How should this be designed??


For now, I am restricting it to only consider 1 config.
It won't listen on multiple ports for multicast traffic. This will be changed once I have the basic structure ready.


For now, Only work on IPv4. IPv6 will be added once IPv4 is ready



#### MDNS

1. If a DNS Query comes on the source interface, We don't forward it to the destination. We want the destination to be able to resolve mdns hosts in source. A Query from source should not be forwarded to the destination.

2. A DNS answer from source should be forwarded to the destination _if_ the domain name is in the allow list for that config.

3. A DNS query from destination should not be forwarded to source if it is not in the allow list for the config

4. A DNS answer should not be forwarded from destination to source in any circumstances






