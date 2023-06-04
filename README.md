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










