# \DefaultApi

All URIs are relative to *https://sessionserver.mojang.com*

Method | HTTP request | Description
------------- | ------------- | -------------
[**get_blocked_servers**](DefaultApi.md#get_blocked_servers) | **GET** /blockedservers | 
[**get_user_by_id**](DefaultApi.md#get_user_by_id) | **GET** /session/minecraft/profile/{userId} | 
[**has_joined_server**](DefaultApi.md#has_joined_server) | **GET** /session/minecraft/hasJoined | 



## get_blocked_servers

> get_blocked_servers()


### Parameters

This endpoint does not need any parameter.

### Return type

 (empty response body)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: Not defined

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_user_by_id

> crate::models::User get_user_by_id(user_id, unsigned)


### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**user_id** | **uuid::Uuid** |  | [required] |
**unsigned** | Option<**bool**> |  |  |[default to true]

### Return type

[**crate::models::User**](User.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## has_joined_server

> crate::models::User has_joined_server(username, server_id, ip)


### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**username** | Option<**String**> |  |  |
**server_id** | Option<**String**> |  |  |
**ip** | Option<**String**> |  |  |

### Return type

[**crate::models::User**](User.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

